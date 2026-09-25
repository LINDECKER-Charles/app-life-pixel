//! The helpers of the stack tests themselves: test databases removed when dropped, stale ones
//! removed by the next, test storage emptied, and test mailboxes that find their messages.

use std::str::FromStr;
use std::time::Duration;

use bytes::Bytes;
use life_pixel_server::config::Config;
use life_pixel_server::storage::object_store;
use life_pixel_server::testing::{TEST_DATABASE_PREFIX, TestDatabase, TestMailbox, TestStorage};
use object_store::ObjectStoreExt;
use object_store::path::Path;
use sqlx::postgres::PgConnectOptions;
use sqlx::{AssertSqlSafe, Connection, PgConnection};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const DATABASE_EXISTS: &str = "select count(*) from pg_database where datname = $1";
/// How long a message may take to reach Mailpit.
const DELIVERY_DELAY: Duration = Duration::from_secs(10);

async fn admin() -> PgConnection {
    life_pixel_server::testing::load_test_env();
    let url = std::env::var("LP_TEST_DATABASE_URL").unwrap();
    PgConnection::connect_with(&PgConnectOptions::from_str(&url).unwrap())
        .await
        .unwrap()
}

async fn exists(admin: &mut PgConnection, name: &str) -> bool {
    let count: i64 = sqlx::query_scalar(DATABASE_EXISTS)
        .bind(name)
        .fetch_one(admin)
        .await
        .unwrap();
    count == 1
}

async fn execute(admin: &mut PgConnection, statement: String) {
    sqlx::raw_sql(AssertSqlSafe(statement))
        .execute(admin)
        .await
        .unwrap();
}

fn random_name() -> String {
    format!("{TEST_DATABASE_PREFIX}{:016x}", rand::random::<u64>())
}

#[tokio::test]
async fn a_test_database_is_removed_when_dropped() {
    let mut admin = admin().await;
    let dropped = TestDatabase::create().await.unwrap();
    let out_of_scope = TestDatabase::create().await.unwrap();
    let names = [dropped.name().to_owned(), out_of_scope.name().to_owned()];
    assert!(exists(&mut admin, &names[0]).await);

    dropped.drop().await.unwrap();
    drop(out_of_scope);

    assert!(!exists(&mut admin, &names[0]).await);
    assert!(!exists(&mut admin, &names[1]).await);
}

#[tokio::test]
async fn a_new_test_database_removes_those_more_than_an_hour_old_only() {
    let mut admin = admin().await;
    let (stale, recent, uncommented) = (random_name(), random_name(), random_name());
    for name in [&stale, &recent, &uncommented] {
        execute(&mut admin, format!("create database {name}")).await;
    }
    let long_ago = "2000-01-01T00:00:00Z";
    execute(
        &mut admin,
        format!("comment on database {stale} is '{long_ago}'"),
    )
    .await;
    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    execute(
        &mut admin,
        format!("comment on database {recent} is '{now}'"),
    )
    .await;

    let database = TestDatabase::create().await.unwrap();

    assert!(!exists(&mut admin, &stale).await);
    assert!(exists(&mut admin, &recent).await);
    assert!(exists(&mut admin, &uncommented).await);
    for name in [recent, uncommented] {
        execute(&mut admin, format!("drop database {name} with (force)")).await;
    }
    database.drop().await.unwrap();
}

#[tokio::test]
async fn test_storage_is_emptied_when_dropped() {
    let storage = TestStorage::create().unwrap();
    let prefix = Path::from(storage.prefix());
    let key = Path::from("documents/some/thing.json");
    storage
        .objects()
        .put(&key, Bytes::from_static(b"{}").into())
        .await
        .unwrap();
    assert_eq!(storage.keys().await.unwrap(), ["documents/some/thing.json"]);

    drop(storage);

    let root = object_store(&Config::from_env().unwrap().storage).unwrap();
    let left: Vec<_> = futures_util::TryStreamExt::try_collect(root.list(Some(&prefix)))
        .await
        .unwrap();
    assert!(left.is_empty(), "{left:?}");
}

#[tokio::test]
async fn a_test_mailbox_finds_the_messages_sent_to_it_only() {
    let mailbox = TestMailbox::new().unwrap();
    let other = TestMailbox::new().unwrap();
    let token = format!("{:016x}", rand::random::<u64>());

    send_mail(
        mailbox.address(),
        &format!("Code {token}"),
        &format!("Your code is {token}."),
    )
    .await;

    let message = mailbox.wait_for_message(DELIVERY_DELAY).await.unwrap();
    assert_eq!(message.subject, format!("Code {token}"));
    assert!(message.text.contains(&token), "{:?}", message.text);
    assert!(other.messages().await.unwrap().is_empty());
}

/// Sends a plain-text message to `to` through the SMTP server of `.env`.
async fn send_mail(to: &str, subject: &str, body: &str) {
    let config = Config::from_env().unwrap();
    let server = config
        .mail
        .smtp_url
        .expose()
        .trim_start_matches("smtp://")
        .to_owned();
    let stream = TcpStream::connect(server).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let message = format!(
        "From: <no-reply@localhost>\r\nTo: <{to}>\r\nSubject: {subject}\r\n\r\n{body}\r\n.\r\n"
    );
    let exchange = [
        String::new(),
        "EHLO localhost\r\n".to_owned(),
        "MAIL FROM:<no-reply@localhost>\r\n".to_owned(),
        format!("RCPT TO:<{to}>\r\n"),
        "DATA\r\n".to_owned(),
        message,
        "QUIT\r\n".to_owned(),
    ];
    for command in exchange {
        writer.write_all(command.as_bytes()).await.unwrap();
        read_reply(&mut reader).await;
    }
}

/// Reads one reply, its continuation lines included, and checks it is not an error.
async fn read_reply(reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>) {
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert!(line.starts_with(['2', '3']), "SMTP error: {line:?}");
        if line.as_bytes().get(3) != Some(&b'-') {
            return;
        }
    }
}
