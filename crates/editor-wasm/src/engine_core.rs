//! The engine as plain Rust: one open animation and its history, every method of the interface,
//! tested on the host. [`bindings`](crate::bindings) only converts values around it.

use life_pixel_core::edit::{History, Operation};
use life_pixel_core::serialize::{read_document, write_document};
use life_pixel_core::{Animation, Limits, NewAnimation};

use crate::errors::EngineError;
use crate::export::{ExportRequest, ExportResult, export_files};
use crate::new_animation::NewAnimationOptions;
use crate::render::{RenderRequest, RenderedFrame, render_frame};
use crate::snippet::{SnippetRequest, render_request};
use crate::state::{DocumentSummary, EngineState, Status};

/// The editor's engine: at most one animation, the undo history of its edits, and whether they are
/// saved. Every rule is `core`'s and every file `compiler`'s; a refused request changes nothing.
#[derive(Clone, Debug, Default)]
pub struct EngineCore {
    animation: Option<Animation>,
    history: History,
    /// Restored from a snapshot of unsaved work: unsaved until the next save, whatever the
    /// history says, since the history of that work is gone.
    is_restored_unsaved: bool,
}

impl EngineCore {
    /// Replaces the open animation, if any, with a new one; its history starts empty and saved.
    ///
    /// # Errors
    ///
    /// The `document.*` code of a size, a duration, a name or a palette out of its limits.
    pub fn create(&mut self, options: NewAnimationOptions) -> Result<(), EngineError> {
        let animation = Animation::new(NewAnimation::try_from(options)?)?;
        self.replace(animation);
        Ok(())
    }

    /// Replaces the open animation, if any, with the one of `document`; its history starts empty
    /// and saved.
    ///
    /// # Errors
    ///
    /// The `document.*` code of the first rule the document breaks.
    pub fn open(&mut self, document: &[u8]) -> Result<(), EngineError> {
        self.replace(read_document(document)?);
        Ok(())
    }

    /// [`open`](Self::open) a snapshot of unsaved work, after the worker failed: the animation
    /// stays unsaved until [`mark_saved`](Self::mark_saved).
    ///
    /// # Errors
    ///
    /// Those of [`open`](Self::open).
    pub fn restore(&mut self, document: &[u8]) -> Result<(), EngineError> {
        self.open(document)?;
        self.is_restored_unsaved = true;
        Ok(())
    }

    /// Applies `operation` and keeps it for undo.
    ///
    /// # Errors
    ///
    /// `engine.no_document` before an animation is open, or the operation's `edit.*`,
    /// `import.*` or `document.*` code.
    pub fn apply(&mut self, operation: Operation) -> Result<(), EngineError> {
        let animation = self
            .animation
            .as_mut()
            .ok_or_else(EngineError::no_document)?;
        self.history.apply(animation, operation)?;
        Ok(())
    }

    /// Undoes the last step, and says whether there was one: otherwise it changes nothing.
    pub fn undo(&mut self) -> bool {
        let animation = self.animation.as_mut();
        animation.is_some_and(|animation| self.history.undo(animation))
    }

    /// Redoes the last undone step, and says whether there was one: otherwise it changes nothing.
    pub fn redo(&mut self) -> bool {
        let animation = self.animation.as_mut();
        animation.is_some_and(|animation| self.history.redo(animation))
    }

    /// Marks the animation, as it is now, saved.
    pub fn mark_saved(&mut self) {
        self.history.mark_saved();
        self.is_restored_unsaved = false;
    }

    /// What the interface shows: the animation's summary, the history's flags and the limits.
    #[must_use]
    pub fn state(&self) -> EngineState {
        let Some(animation) = &self.animation else {
            return EngineState {
                status: Status::Empty,
                document: None,
                can_undo: false,
                can_redo: false,
                has_unsaved_work: false,
                limits: Self::limits(),
            };
        };
        EngineState {
            status: Status::Ready,
            document: Some(DocumentSummary::from(animation)),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            has_unsaved_work: self.is_restored_unsaved || !self.history.is_saved(),
            limits: Self::limits(),
        }
    }

    /// A frame as RGBA, with an operation in progress previewed on it, without changing anything.
    ///
    /// # Errors
    ///
    /// `engine.no_document`, `edit.frame_not_found`, or the error of the previewed operation.
    pub fn render(&self, request: &RenderRequest) -> Result<RenderedFrame, EngineError> {
        render_frame(self.document()?, request)
    }

    /// The animation as a document, the bytes `open` reads back.
    ///
    /// # Errors
    ///
    /// `engine.no_document`, or `document.too_large` for a document that could not be read back.
    pub fn serialize(&self) -> Result<Vec<u8>, EngineError> {
        Ok(write_document(self.document()?)?.into_bytes())
    }

    /// The files of an export.
    ///
    /// # Errors
    ///
    /// `engine.no_document`, or the compiler's code.
    pub fn export(&self, request: &ExportRequest) -> Result<ExportResult, EngineError> {
        export_files(self.document()?, request)
    }

    /// The integration snippet of the animation's export.
    ///
    /// # Errors
    ///
    /// `engine.no_document`.
    pub fn snippet(&self, request: &SnippetRequest) -> Result<String, EngineError> {
        Ok(render_request(self.document()?, request))
    }

    /// The limits of this build: the interface never repeats one.
    #[must_use]
    pub const fn limits() -> Limits {
        Limits::current()
    }

    fn document(&self) -> Result<&Animation, EngineError> {
        self.animation.as_ref().ok_or_else(EngineError::no_document)
    }

    fn replace(&mut self, animation: Animation) {
        *self = Self {
            animation: Some(animation),
            ..Self::default()
        };
    }
}
