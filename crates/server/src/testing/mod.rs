//! What the tests that need the local stack use, feature `stack-tests`: a database, a bucket
//! prefix and a mail recipient of their own, so that parallel runs — two worktrees on one stack
//! included — never meet.
//!
//! The stack is found through the repository's `.env`, copied from `.env.example`:
//! `LP_TEST_DATABASE_URL` for [`TestDatabase`], `LP_STORAGE_URL` and `LP_S3_*` for
//! [`TestStorage`], `LP_TEST_MAILPIT_URL` for [`TestMailbox`].

mod background;
mod database;
mod env;
mod mailbox;
mod storage;

pub use database::{TEST_DATABASE_OWNER, TEST_DATABASE_PREFIX, TestDatabase};
pub use env::{TestSetupError, load_test_env};
pub use mailbox::{MailMessage, TEST_MAIL_DOMAIN, TestMailbox};
pub use storage::TestStorage;
