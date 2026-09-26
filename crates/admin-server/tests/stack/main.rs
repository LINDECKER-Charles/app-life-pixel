//! The admin server over the local stack, feature `stack-tests`: each test on a database of its
//! own, `lpa_test_…`, owned by `life_pixel_admin` — never `life_pixel_admin` itself. Sign-in,
//! TOTP replays, idle and absolute expiry on a clock the test moves, CSRF, disabling, and the
//! `create-admin` and `disable-admin` commands.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod commands;
#[path = "../common/mod.rs"]
mod common;
mod database;
mod sessions;
mod support;
