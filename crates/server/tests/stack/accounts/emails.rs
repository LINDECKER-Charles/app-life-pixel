//! The emails, through Mailpit: in the account's language, plain text and HTML, with links that
//! work once and expire.

use life_pixel_server::testing::{MailMessage, TestMailbox};
use serde_json::json;

use super::{AccountsStack, message, messages, post, text, token_in};
use crate::router::auth::{APP_ORIGIN, Browser, NEW_PASSWORD, PASSWORD, sign_in};

const VERIFY_LIFETIME: &str = "select extract(epoch from expires_at - created_at)::bigint from \
                               email_tokens where purpose = 'verify_email' and account_id = \
                               (select id from accounts where email = $1::citext)";
const RESET_LIFETIME: &str = "select extract(epoch from expires_at - created_at)::bigint from \
                              email_tokens where purpose = 'reset_password' and account_id = \
                              (select id from accounts where email = $1::citext)";
const EXPIRE_TOKENS: &str = "update email_tokens set expires_at = now() - interval '1 second' \
                             where account_id = (select id from accounts where email = \
                             $1::citext)";

/// The plain text of `message`, with its line ends as the catalogues write them.
fn plain(message: &MailMessage) -> String {
    message.text.replace("\r\n", "\n").trim_end().to_owned()
}

/// Checks that `email` is the verification email in `language`, text and HTML: its token.
fn verification_token(email: &MailMessage, language: &str) -> String {
    let token = token_in(email, "/verify-email");
    let link = format!("{APP_ORIGIN}/verify-email?token={token}");
    let body = text(language, "email.verify_email.body").replace("{link}", &link);
    assert_eq!(plain(email), body);
    assert!(email.html.contains(&format!("<html lang=\"{language}\">")));
    token
}

/// Checks the verification email of an account signed up in `language`, and its link.
async fn verification_in(language: &str) {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    let browser = stack.sign_up(&mailbox, language).await;
    let email = message(&mailbox, &text(language, "email.verify_email.subject")).await;
    let token = verification_token(&email, language);
    let verify = post("verify-email", &json!({ "token": token }));
    assert_eq!(stack.status(verify).await, 204);
    let session = stack.send(browser.get("/api/v1/auth/session")).await;
    assert_eq!(session.json()["account"]["emailVerified"], true);
    let seconds: i64 = stack.select(VERIFY_LIFETIME, mailbox.address()).await;
    assert_eq!(seconds, 7 * 24 * 60 * 60);
}

/// Checks the reset email of an account signed up in `language`, its link, and the email that
/// follows a reset, then a change.
async fn reset_in(language: &str) {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    stack.sign_up(&mailbox, language).await;
    let token = stack.reset_token(&mailbox, language).await;
    let seconds: i64 = stack.select(RESET_LIFETIME, mailbox.address()).await;
    assert_eq!(seconds, 60 * 60);
    let body = json!({ "token": token, "password": PASSWORD });
    assert_eq!(
        stack.status(post("password-reset/confirm", &body)).await,
        204
    );
    let subject = text(language, "email.password_changed.subject");
    password_changed(&message(&mailbox, &subject).await, language);
    change_in(&stack, &mailbox, &subject).await;
}

/// Checks that `email` is the email saying the password changed, in `language`.
fn password_changed(email: &MailMessage, language: &str) {
    assert_eq!(plain(email), text(language, "email.password_changed.body"));
    assert!(email.html.contains(&format!("<html lang=\"{language}\">")));
}

/// Checks that changing the password of `mailbox`'s account sends the email of `subject` again.
async fn change_in(stack: &AccountsStack, mailbox: &TestMailbox, subject: &str) {
    let browser = Browser::of(&stack.send(sign_in(mailbox.address(), PASSWORD)).await);
    let change = browser.change_password(PASSWORD, NEW_PASSWORD);
    assert_eq!(stack.status(change).await, 204);
    assert_eq!(messages(mailbox, subject, 2).await.len(), 2);
}

#[tokio::test]
async fn the_verification_email_is_in_english_with_a_working_link() {
    verification_in("en").await;
}

#[tokio::test]
async fn the_verification_email_is_in_french_with_a_working_link() {
    verification_in("fr").await;
}

#[tokio::test]
async fn the_reset_emails_are_in_english_with_a_working_link() {
    reset_in("en").await;
}

#[tokio::test]
async fn the_reset_emails_are_in_french_with_a_working_link() {
    reset_in("fr").await;
}

#[tokio::test]
async fn a_link_works_once_and_not_once_expired() {
    let stack = AccountsStack::new().await;
    let mailbox = TestMailbox::new().unwrap();
    stack.sign_up(&mailbox, "en").await;
    let verify = message(&mailbox, &text("en", "email.verify_email.subject")).await;
    let body = json!({ "token": token_in(&verify, "/verify-email") });
    assert_eq!(stack.status(post("verify-email", &body)).await, 204);
    let again = stack.send(post("verify-email", &body)).await;
    again.assert_problem(400, "auth.token_invalid");
    let token = stack.reset_token(&mailbox, "en").await;
    assert!(stack.execute(EXPIRE_TOKENS, mailbox.address()).await >= 1);
    let body = json!({ "token": token, "password": NEW_PASSWORD });
    let late = stack.send(post("password-reset/confirm", &body)).await;
    late.assert_problem(400, "auth.token_invalid");
    let (_, tokens) = stack.state.accounts.purge_expired().await.unwrap();
    assert!(tokens >= 1, "the purge removes expired tokens");
}
