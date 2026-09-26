//! The library watcher (desktop.md, T3): what an agent or the CLI changes in the library folder
//! reaches the app as `library-changed`, with the ids concerned, once the folder has been quiet
//! for 300 ms.

mod change;

pub use change::LibraryChange;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use notify_debouncer_full::notify::{self, EventKind, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use tauri::{AppHandle, Emitter, Runtime};

/// The event the app listens to, carrying a [`LibraryChange`].
pub const LIBRARY_CHANGED: &str = "library-changed";
/// How long the folder stays quiet before its changes are reported together.
const DEBOUNCE: Duration = Duration::from_millis(300);

/// Where the changes go: the app's webview, or a test's channel.
type Listener = Arc<dyn Fn(LibraryChange) + Send + Sync>;
/// The debounced watcher of one folder; dropping it stops watching.
type FolderDebouncer = Debouncer<RecommendedWatcher, RecommendedCache>;

/// Watches one library folder at a time and reports its changes to a listener.
pub struct LibraryWatcher {
    listener: Listener,
    debouncer: Mutex<Option<FolderDebouncer>>,
}

impl LibraryWatcher {
    /// A watcher reporting to `listener`, watching nothing yet.
    #[must_use]
    pub fn new(listener: impl Fn(LibraryChange) + Send + Sync + 'static) -> Self {
        Self {
            listener: Arc::new(listener),
            debouncer: Mutex::new(None),
        }
    }

    /// Watches `root` instead of the folder watched until now. A folder that cannot be watched
    /// leaves the app without its changes, which the log says.
    pub fn watch(&self, root: &Path) {
        let mut debouncer = self
            .debouncer
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *debouncer = None;
        match self.start(root) {
            Ok(started) => *debouncer = Some(started),
            Err(error) => tracing::warn!(path = %root.display(), %error, "library not watched"),
        }
    }

    /// A debouncer over the whole of `root`, reporting what its events concern.
    fn start(&self, root: &Path) -> notify::Result<FolderDebouncer> {
        let roots = Roots::of(root);
        let listener = Arc::clone(&self.listener);
        let mut debouncer = new_debouncer(DEBOUNCE, None, move |result| {
            if let Some(change) = roots.change_of(result) {
                listener(change);
            }
        })?;
        debouncer.watch(root, RecursiveMode::Recursive)?;
        Ok(debouncer)
    }
}

/// A listener sending each change to the app's webview as `library-changed`.
pub fn emitter<R: Runtime>(app: AppHandle<R>) -> impl Fn(LibraryChange) + Send + Sync + 'static {
    move |change| {
        if let Err(error) = app.emit(LIBRARY_CHANGED, change) {
            tracing::warn!(%error, "library change not sent");
        }
    }
}

/// The library folder as given and as the system reports it: macOS names `/var` `/private/var`.
struct Roots(Vec<PathBuf>);

impl Roots {
    fn of(root: &Path) -> Self {
        let mut roots = vec![root.to_path_buf()];
        let canonical = root
            .canonicalize()
            .ok()
            .filter(|canonical| canonical != root);
        roots.extend(canonical);
        Self(roots)
    }

    /// What a batch of events concerns; `None` when nothing the app shows. A watcher that lost
    /// track reports an empty change: anything may have changed.
    fn change_of(&self, result: DebounceEventResult) -> Option<LibraryChange> {
        let events = match result {
            Ok(events) => events,
            Err(errors) => {
                tracing::warn!(?errors, "library watcher failed");
                return Some(LibraryChange::default());
            }
        };
        let mut change = LibraryChange::default();
        let writes = events
            .iter()
            .filter(|event| !matches!(event.kind, EventKind::Access(_)));
        for event in writes {
            if event.need_rescan() {
                return Some(LibraryChange::default());
            }
            let inside = event.paths.iter().filter_map(|path| self.relative(path));
            inside.for_each(|relative| change.record(relative));
        }
        (!change.is_empty()).then_some(change)
    }

    /// `path` relative to the library folder, when inside it.
    fn relative<'a>(&self, path: &'a Path) -> Option<&'a Path> {
        self.0.iter().find_map(|root| path.strip_prefix(root).ok())
    }
}
