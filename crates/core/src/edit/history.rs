use std::collections::VecDeque;

use super::{EditError, Inverse, Operation, apply};
use crate::limits::{HISTORY_MAX_BYTES, HISTORY_MAX_STEPS};
use crate::model::Animation;

/// A state of the animation the history can tell apart: 0 before any step, then one number per
/// applied step.
type State = u64;

/// The state before any step.
const INITIAL_STATE: State = 0;

/// The undo and redo stacks of one animation.
///
/// Beyond [`HISTORY_MAX_STEPS`] steps or [`HISTORY_MAX_BYTES`] of inverses, the oldest steps are
/// dropped. A new history counts its animation as saved.
#[derive(Clone, Debug)]
pub struct History {
    undo: VecDeque<Step>,
    redo: Vec<Step>,
    /// The bytes of the inverses on the undo stack.
    undo_bytes: usize,
    /// The state once every step on the undo stack is undone.
    base: State,
    /// The number of the last step applied.
    last_step: State,
    /// The state marked saved.
    saved: State,
}

impl Default for History {
    /// An empty history, its animation saved as it is.
    fn default() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            undo_bytes: 0,
            base: INITIAL_STATE,
            last_step: INITIAL_STATE,
            saved: INITIAL_STATE,
        }
    }
}

/// An applied operation: what undoes it — or, on the redo stack, what redoes it —, and the state
/// it leads to.
#[derive(Clone, Debug)]
struct Step {
    inverse: Inverse,
    after: State,
}

impl History {
    /// Applies `operation` to `animation` and keeps its inverse; clears the redo stack. An
    /// operation that changes nothing leaves the history as it was.
    ///
    /// # Errors
    ///
    /// The operation's [`EditError`]; both the animation and the history are then left as they
    /// were.
    pub fn apply(
        &mut self,
        animation: &mut Animation,
        operation: Operation,
    ) -> Result<(), EditError> {
        let inverse = apply(animation, &operation)?;
        if inverse.is_empty() {
            return Ok(());
        }
        self.redo.clear();
        self.last_step += 1;
        let step = Step {
            inverse,
            after: self.last_step,
        };
        self.push_undo(step);
        Ok(())
    }

    /// Undoes the last step on `animation`; false when there is none.
    pub fn undo(&mut self, animation: &mut Animation) -> bool {
        let Some(step) = self.undo.pop_back() else {
            return false;
        };
        self.undo_bytes -= step.inverse.byte_size();
        self.redo.push(Step {
            inverse: step.inverse.apply(animation),
            ..step
        });
        true
    }

    /// Redoes the last undone step on `animation`; false when there is none.
    pub fn redo(&mut self, animation: &mut Animation) -> bool {
        let Some(step) = self.redo.pop() else {
            return false;
        };
        self.push_undo(Step {
            inverse: step.inverse.apply(animation),
            ..step
        });
        true
    }

    /// Whether there is a step to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is a step to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Marks the animation's current state as saved.
    pub fn mark_saved(&mut self) {
        self.saved = self.current();
    }

    /// Whether the animation is back to the state marked saved.
    #[must_use]
    pub fn is_saved(&self) -> bool {
        self.saved == self.current()
    }

    fn current(&self) -> State {
        self.undo.back().map_or(self.base, |step| step.after)
    }

    /// Keeps `step` for undo, then drops the oldest steps beyond the limits.
    fn push_undo(&mut self, step: Step) {
        self.undo_bytes += step.inverse.byte_size();
        self.undo.push_back(step);
        while self.undo.len() > HISTORY_MAX_STEPS || self.undo_bytes > HISTORY_MAX_BYTES {
            let Some(oldest) = self.undo.pop_front() else {
                break;
            };
            self.undo_bytes -= oldest.inverse.byte_size();
            self.base = oldest.after;
        }
    }
}
