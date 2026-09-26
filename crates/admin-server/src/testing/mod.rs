//! What the tests that need the local stack use, feature `stack-tests`: a database of their own,
//! so that parallel runs — two worktrees on one stack included — never meet, and never touch
//! `life_pixel_admin` itself.
//!
//! The stack is found through the repository's `.env`, copied from `.env.example`:
//! `LP_TEST_DATABASE_URL`, shared with the server's tests, for [`TestDatabase`].

mod database;
mod env;

pub use database::{TEST_DATABASE_OWNER, TEST_DATABASE_PREFIX, TestDatabase};
pub use env::{TestSetupError, load_test_env};
