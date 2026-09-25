//! Signing in with an unknown address takes as long as with a wrong password: both spend one
//! Argon2id hash, so timing does not tell whether an address has an account.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::Request;
use life_pixel_server::testing::{TEST_MAIL_DOMAIN, TestMailbox};

use super::AccountsStack;
use crate::router::auth::{PASSWORD, peer, sign_in};

/// The attempts of each kind: the sign-in limit allows five per address signed in to.
const ROUNDS: u8 = 5;

/// How long `stack` takes to refuse `request` from `peer`.
async fn refusal(stack: &AccountsStack, peer: SocketAddr, request: Request<Body>) -> Duration {
    let started = Instant::now();
    let answer = stack.send_from(peer, request).await;
    let elapsed = started.elapsed();
    answer.assert_problem(401, "auth.invalid_credentials");
    elapsed
}

/// The median of `durations`.
fn median(mut durations: Vec<Duration>) -> Duration {
    durations.sort();
    durations[durations.len() / 2]
}

#[tokio::test]
async fn an_unknown_address_takes_as_long_as_a_wrong_password() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    stack.sign_up(&mailbox, "en").await;
    let warm_up = sign_in(&format!("warm-up@{TEST_MAIL_DOMAIN}"), PASSWORD);
    refusal(&stack, peer(100), warm_up).await;
    let (mut unknown, mut known) = (Vec::new(), Vec::new());
    for n in 0..ROUNDS {
        let absent = sign_in(&format!("nobody-{n}@{TEST_MAIL_DOMAIN}"), PASSWORD);
        unknown.push(refusal(&stack, peer(2 * n + 1), absent).await);
        let wrong = sign_in(mailbox.address(), "wrong password!");
        known.push(refusal(&stack, peer(2 * n + 2), wrong).await);
    }
    let (unknown, known) = (median(unknown), median(known));
    let alike = unknown * 2 > known && known * 2 > unknown;
    assert!(
        alike,
        "unknown address {unknown:?}, wrong password {known:?}"
    );
}
