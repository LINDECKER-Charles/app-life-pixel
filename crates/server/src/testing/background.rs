//! Cleaning up from `Drop`, which cannot await: the cleanup runs to its end on a thread of its
//! own, with a runtime and connections of its own — never those of the test's runtime, which
//! waits for it.

use std::future::Future;

/// Runs `cleanup` to its end on a new thread; a failure is logged.
pub(super) fn run_to_end<F>(what: &'static str, cleanup: impl FnOnce() -> F + Send + 'static)
where
    F: Future<Output = Result<(), super::TestSetupError>>,
{
    let thread = std::thread::spawn(move || block_on(cleanup()));
    let error = match thread.join() {
        Ok(Ok(())) => return,
        Ok(Err(error)) => error.to_string(),
        Err(_) => "the cleanup panicked".to_owned(),
    };
    tracing::warn!(%error, what, "a test resource was not cleaned up");
}

/// Runs `cleanup` on a runtime of its own.
fn block_on<F>(cleanup: F) -> std::io::Result<()>
where
    F: Future<Output = Result<(), super::TestSetupError>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(cleanup).map_err(std::io::Error::other)
}
