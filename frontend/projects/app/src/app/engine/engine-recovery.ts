import type { EditorEngine } from './editor-engine';

/** What an engine reports once it restarted after a failure (editor.md, W1). */
export interface RecoveryReport {
  /** The changes made since the snapshot it reopened, which are lost. */
  readonly lostChanges: number;
}

/**
 * An engine that recovers from failures of its own: `WasmEditorEngine`, whose worker can fail.
 * It is not part of `EditorEngine`, which the mock implements as it is.
 */
export interface EngineRecovery {
  /** Calls `listener` after each recovery; returns the unsubscribe function. */
  onRecovered(listener: (report: RecoveryReport) => void): () => void;
}

export function canRecover(engine: EditorEngine): engine is EditorEngine & EngineRecovery {
  return typeof (engine as Partial<EngineRecovery>).onRecovered === 'function';
}
