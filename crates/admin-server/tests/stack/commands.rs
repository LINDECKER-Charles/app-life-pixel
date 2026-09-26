//! `create-admin --password-stdin` and `disable-admin`, run as the binary is, on a test database.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use axum::http::StatusCode;
use life_pixel_admin_server::admins::secret_box::SecretBox;
use life_pixel_admin_server::admins::totp::{TotpSecret, step_at};
use life_pixel_admin_server::config::{FromVariable, Key};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::common::TOTP_KEY;
use crate::support::{PASSWORD, Stack};

const URI_PREFIX: &str = "otpauth://totp/Life%20Pixel:ops%40example.org?secret=";
const URI_SUFFIX: &str = "&issuer=Life%20Pixel";

/// Runs the binary with `arguments` on `database_url`, `stdin` on its standard input; in an
/// empty environment, so that no `.env` and no real database is ever read.
fn run(database_url: &str, arguments: &[&str], stdin: &str) -> Output {
    let folder = std::env::temp_dir();
    let mut child = Command::new(env!("CARGO_BIN_EXE_life-pixel-admin-server"))
        .args(arguments)
        .env_clear()
        .env("LPA_ENVIRONMENT", "local")
        .env("LPA_DATABASE_URL", database_url)
        .env("LPA_TOTP_KEY", TOTP_KEY)
        .current_dir(folder)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn create(database_url: &str, email: &str, password: &str) -> Output {
    let arguments = ["create-admin", email, "--password-stdin"];
    run(database_url, &arguments, &format!("{password}\n"))
}

#[tokio::test]
async fn create_admin_prints_the_uri_of_the_secret_it_seals() {
    let stack = Stack::new().await;
    let url = stack.database.url().unwrap();
    let output = create(&url, "ops@example.org", PASSWORD);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let uri = String::from_utf8(output.stdout).unwrap();
    let secret = uri
        .trim_end()
        .strip_prefix(URI_PREFIX)
        .and_then(|rest| rest.strip_suffix(URI_SUFFIX))
        .unwrap_or_else(|| panic!("{uri}"));
    assert_eq!(secret.len(), 32, "20 bytes in base32");
    let (id, sealed): (Uuid, Vec<u8>) = sqlx::query_as("select id, totp_secret from admins")
        .fetch_one(stack.database.pool())
        .await
        .unwrap();
    let key = Key::from_variable(TOTP_KEY).unwrap();
    let stored: TotpSecret = SecretBox::new(&key).open(id, &sealed).unwrap();
    assert_eq!(stored.to_base32(), secret);
    let code = stored.code_at(step_at(OffsetDateTime::now_utc()));
    let (status, _, _) = stack.sign_in(("ops@example.org", PASSWORD, &code)).await;
    assert_eq!(status, StatusCode::OK);
    stack.drop().await;
}

#[tokio::test]
async fn create_admin_refuses_a_short_password_and_a_taken_address() {
    let stack = Stack::new().await;
    let url = stack.database.url().unwrap();
    assert!(!create(&url, "ops@example.org", "short").status.success());
    assert!(!create(&url, "not an address", PASSWORD).status.success());
    assert!(create(&url, "ops@example.org", PASSWORD).status.success());
    let again = create(&url, "OPS@example.org", PASSWORD);
    assert!(!again.status.success());
    assert!(again.stdout.is_empty(), "no URI for a refused admin");
    let (count,): (i64,) = sqlx::query_as("select count(*) from admins")
        .fetch_one(stack.database.pool())
        .await
        .unwrap();
    assert_eq!(count, 1);
    stack.drop().await;
}

#[tokio::test]
async fn disable_admin_succeeds_once() {
    let stack = Stack::new().await;
    let url = stack.database.url().unwrap();
    assert!(create(&url, "ops@example.org", PASSWORD).status.success());
    assert!(
        run(&url, &["disable-admin", "ops@example.org"], "")
            .status
            .success()
    );
    assert!(
        !run(&url, &["disable-admin", "ops@example.org"], "")
            .status
            .success()
    );
    stack.drop().await;
}
