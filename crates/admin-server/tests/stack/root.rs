//! The root admin of `LPA_ROOT_ADMIN_*`: created when there is no admin at all, never beside
//! another one — active or disabled —, and signed in to with the operator's own TOTP secret.

use axum::http::StatusCode;
use life_pixel_admin_server::admins::clock::Clock;
use life_pixel_admin_server::admins::password::Password;
use life_pixel_admin_server::admins::sign_in::AdminAccount;
use life_pixel_admin_server::admins::totp::{TotpSecret, step_at};

use crate::support::{PASSWORD, Stack};

const ROOT_EMAIL: &str = "root@example.org";

/// A root admin of `email`, with [`PASSWORD`] and a new secret.
fn root(email: &str) -> AdminAccount {
    AdminAccount {
        email: email.to_owned(),
        password: Password::new(PASSWORD.to_owned()).unwrap(),
        secret: TotpSecret::generate(),
    }
}

#[tokio::test]
async fn the_root_admin_is_created_on_an_empty_database_and_signs_in_with_its_secret() {
    let stack = Stack::new().await;
    let root = root(ROOT_EMAIL);
    let created = stack.state.admins.create_root(&root).await.unwrap();
    assert_eq!(created.unwrap().email, ROOT_EMAIL);
    let code = root.secret.code_at(step_at(stack.clock.now()));
    let (status, body, cookie) = stack.sign_in((ROOT_EMAIL, PASSWORD, &code)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(cookie.is_some());
    stack.drop().await;
}

#[tokio::test]
async fn the_root_admin_is_created_once_even_by_two_servers_starting_together() {
    let stack = Stack::new().await;
    let admins = &stack.state.admins;
    let (first, second) = (root(ROOT_EMAIL), root("other@example.org"));
    let (first, second) = tokio::join!(admins.create_root(&first), admins.create_root(&second));
    assert_eq!(
        [first.unwrap(), second.unwrap()].iter().flatten().count(),
        1
    );
    let again = admins.create_root(&root(ROOT_EMAIL)).await.unwrap();
    assert!(again.is_none());
    stack.drop().await;
}

#[tokio::test]
async fn no_root_admin_beside_a_disabled_admin() {
    let stack = Stack::new().await;
    let admin = stack.admin("ops@example.org").await;
    let admins = &stack.state.admins;
    assert!(admins.disable(&admin.identity.email).await.unwrap());
    let created = admins.create_root(&root(ROOT_EMAIL)).await.unwrap();
    assert!(created.is_none());
    stack.drop().await;
}
