//! Every command, called directly with a test state over a temporary library; and a few through
//! the IPC, as the app calls them.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

mod animations;
mod export;
mod harness;
mod ipc;
mod platform;
mod projects;
mod settings;
