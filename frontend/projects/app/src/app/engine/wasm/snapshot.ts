/** How long changes go without a snapshot (editor.md, W1: every 30 seconds of changes). */
export const SNAPSHOT_INTERVAL_MS = 30_000;

/** A serialized document, as the page keeps it for a new worker to reopen. */
export interface Snapshot {
  /** The document, or null when none was open. */
  readonly document: Uint8Array | null;
  /** Whether it had unsaved work: a new worker then restores it unsaved. */
  readonly hasUnsavedWork: boolean;
}

/** What a worker failure leaves: the last snapshot, and the changes made since it. */
export interface Recovery {
  readonly snapshot: Snapshot;
  readonly lostChanges: number;
}

const NO_SNAPSHOT: Snapshot = { document: null, hasUnsavedWork: false };

/**
 * The recovery snapshot: the adapter keeps one after `create`, `open` and every save, and asks
 * for one, through `take`, once changes have gone `intervalMs` without it.
 */
export class RecoverySnapshot {
  private snapshot = NO_SNAPSHOT;
  private changes = 0;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private readonly take: () => void;
  private readonly intervalMs: number;

  constructor(take: () => void, intervalMs = SNAPSHOT_INTERVAL_MS) {
    this.take = take;
    this.intervalMs = intervalMs;
  }

  /** Keeps `snapshot`: the changes it holds are no longer at risk. */
  keep(snapshot: Snapshot): void {
    this.snapshot = snapshot;
    this.changes = 0;
    this.cancel();
  }

  /** Counts a change; the first one since the last snapshot starts the interval. */
  recordChange(): void {
    this.changes += 1;
    this.timer ??= setTimeout(() => {
      this.timer = undefined;
      this.take();
    }, this.intervalMs);
  }

  /** The snapshot to reopen and the changes it misses; a new worker starts from it. */
  recover(): Recovery {
    const recovery = { snapshot: this.snapshot, lostChanges: this.changes };
    this.changes = 0;
    this.cancel();
    return recovery;
  }

  dispose(): void {
    this.cancel();
  }

  private cancel(): void {
    clearTimeout(this.timer);
    this.timer = undefined;
  }
}
