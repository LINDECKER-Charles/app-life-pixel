import type { MockDocument } from './mock-state';

/** Snapshot-based undo and redo: simpler than inverse operations, and enough for a mock. */
export interface MockHistory {
  readonly steps: readonly MockDocument[];
  readonly index: number;
  readonly savedIndex: number;
}

export function initialHistory(document: MockDocument): MockHistory {
  return { steps: [document], index: 0, savedIndex: 0 };
}

/** Applying clears the redo branch (core.md, K2). */
export function pushHistory(history: MockHistory, document: MockDocument): MockHistory {
  const steps = [...history.steps.slice(0, history.index + 1), document];
  return { ...history, steps, index: steps.length - 1 };
}

export function undoHistory(history: MockHistory): MockHistory {
  return { ...history, index: Math.max(0, history.index - 1) };
}

export function redoHistory(history: MockHistory): MockHistory {
  return { ...history, index: Math.min(history.steps.length - 1, history.index + 1) };
}

export function markHistorySaved(history: MockHistory): MockHistory {
  return { ...history, savedIndex: history.index };
}

export function currentDocument(history: MockHistory): MockDocument {
  return history.steps[history.index];
}

export function canUndo(history: MockHistory): boolean {
  return history.index > 0;
}

export function canRedo(history: MockHistory): boolean {
  return history.index < history.steps.length - 1;
}

/** True when the document is back to the state `markSaved` last marked (core.md's `is_saved`). */
export function hasUnsavedWork(history: MockHistory): boolean {
  return history.index !== history.savedIndex;
}
