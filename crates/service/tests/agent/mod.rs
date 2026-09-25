//! One file per group of use cases, over the fixture of [`support`].

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

mod conflicts;
mod describe;
mod drawing;
mod frames;
mod outputs;
mod palette;
mod preview;
mod support;
