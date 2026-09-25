import type { EditorEngine } from '../editor-engine';
import type { EngineRecovery, RecoveryReport } from '../engine-recovery';
import type {
  EditOperation,
  EngineState,
  ExportRequest,
  ExportResult,
  NewAnimationOptions,
  RenderedFrame,
  RenderRequest,
  SnippetRequest,
} from '../engine-types';
import { EngineConnection, INTERNAL_ERROR } from './engine-connection';
import type { EngineArgs, EngineMethod, EngineValue } from './protocol';
import { RecoverySnapshot, type Snapshot } from './snapshot';

const STARTING: EngineState = {
  status: 'starting',
  document: null,
  canUndo: false,
  canRedo: false,
  hasUnsavedWork: false,
  limits: null,
};

/** The engine's worker, which esbuild bundles from engine.worker.ts. */
export function createEngineWorker(): Worker {
  return new Worker(new URL('./engine.worker', import.meta.url), { type: 'module' });
}

export interface WasmEditorEngineOptions {
  /** The worker to run the engine in; tests wrap `createEngineWorker` to reach it. */
  readonly createWorker?: () => Worker;
}

/**
 * The `EditorEngine` of the app (editor.md, W1): editor-wasm in a Web Worker. Requests post
 * their buffers transferred — `open`'s document and an import's PNG are detached on this side —
 * and the answers' buffers come back the same way.
 *
 * When the worker fails, a new one reopens the recovery snapshot: the state goes `failed`, then
 * `ready`, and the recovery listeners hear how many changes were lost. A worker that fails before
 * answering anything would fail again: the engine then stays `failed`, and every request rejects
 * with `internal.error`.
 */
export class WasmEditorEngine implements EditorEngine, EngineRecovery {
  private readonly createWorker: () => Worker;
  private readonly snapshot: RecoverySnapshot;
  private readonly stateListeners = new Set<(state: EngineState) => void>();
  private readonly recoveryListeners = new Set<(report: RecoveryReport) => void>();
  private state = STARTING;
  private connection: EngineConnection | null;

  constructor(options: WasmEditorEngineOptions = {}) {
    this.createWorker = options.createWorker ?? createEngineWorker;
    this.snapshot = new RecoverySnapshot(() => void this.takeSnapshot());
    this.connection = this.connect();
    this.request('state', []).then(
      (state) => this.publish(state),
      () => undefined, // the connection reports the failure
    );
  }

  async create(options: NewAnimationOptions): Promise<void> {
    await this.request('create', [options]);
    void this.takeSnapshot();
  }

  async open(document: Uint8Array): Promise<void> {
    await this.request('open', [document]);
    void this.takeSnapshot();
  }

  async apply(operation: EditOperation): Promise<void> {
    await this.request('apply', [operation]);
    this.snapshot.recordChange();
  }

  async undo(): Promise<void> {
    if (await this.request('undo', [])) this.snapshot.recordChange();
  }

  async redo(): Promise<void> {
    if (await this.request('redo', [])) this.snapshot.recordChange();
  }

  async markSaved(): Promise<void> {
    await this.request('markSaved', []);
    void this.takeSnapshot();
  }

  render(request: RenderRequest): Promise<RenderedFrame> {
    return this.request('render', [request]);
  }

  serialize(): Promise<Uint8Array> {
    return this.request('serialize', []);
  }

  export(request: ExportRequest): Promise<ExportResult> {
    return this.request('export', [request]);
  }

  snippet(request: SnippetRequest): Promise<string> {
    return this.request('snippet', [request]);
  }

  subscribe(listener: (state: EngineState) => void): () => void {
    this.stateListeners.add(listener);
    listener(this.state);
    return () => this.stateListeners.delete(listener);
  }

  onRecovered(listener: (report: RecoveryReport) => void): () => void {
    this.recoveryListeners.add(listener);
    return () => this.recoveryListeners.delete(listener);
  }

  /** Stops the worker; every later request rejects with `internal.error`. */
  dispose(): void {
    this.snapshot.dispose();
    this.connection?.close();
    this.connection = null;
  }

  private request<M extends EngineMethod>(method: M, args: EngineArgs<M>): Promise<EngineValue<M>> {
    if (!this.connection) return Promise.reject(INTERNAL_ERROR);
    return this.connection.request(method, args);
  }

  /**
   * Keeps the document as it is now. Answers come in order, so the state last pushed is the one
   * of the serialized document.
   */
  private async takeSnapshot(): Promise<void> {
    try {
      const document = await this.request('serialize', []);
      this.snapshot.keep({ document, hasUnsavedWork: this.state.hasUnsavedWork });
    } catch {
      // The worker failed or the document is gone: the last snapshot stays.
    }
  }

  private connect(): EngineConnection {
    return new EngineConnection(this.createWorker(), {
      state: (state) => this.publish(state),
      failed: (hadAnswered) => this.recover(hadAnswered),
    });
  }

  private recover(hadAnswered: boolean): void {
    const { snapshot, lostChanges } = this.snapshot.recover();
    this.publish({ ...this.state, status: 'failed' });
    if (!hadAnswered) {
      this.connection = null;
      return;
    }
    const connection = this.connect();
    this.connection = connection;
    this.reopen(connection, snapshot).then(
      () => this.announce({ lostChanges }),
      () => undefined, // the new connection reports its own failure
    );
  }

  private async reopen(connection: EngineConnection, snapshot: Snapshot): Promise<void> {
    if (!snapshot.document) {
      this.publish(await connection.request('state', []));
      return;
    }
    // A copy, so that the snapshot outlives the transfer and serves another recovery.
    const document = snapshot.document.slice();
    await connection.request(snapshot.hasUnsavedWork ? 'restore' : 'open', [document]);
  }

  private announce(report: RecoveryReport): void {
    for (const listener of this.recoveryListeners) listener(report);
  }

  private publish(state: EngineState): void {
    this.state = state;
    for (const listener of this.stateListeners) listener(state);
  }
}
