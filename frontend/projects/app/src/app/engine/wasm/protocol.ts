/*
 * The messages between WasmEditorEngine and engine.worker.ts (editor.md, W1): `{ id, method, args }`
 * requests, answered in order by `{ id, ok, value }` or `{ id, ok: false, error }`, and
 * `{ type: 'state', state }` pushed after each command, before its answer.
 */
import type { EditorEngine } from '../editor-engine';
import type { EngineError, EngineState } from '../engine-types';

/**
 * What the worker answers beyond `EditorEngine`: the first state of a worker, a snapshot of
 * unsaved work to restore, and whether an undo or a redo found a step — the adapter counts the
 * changes a failure would lose.
 */
interface WorkerOnlyMethods {
  state(): Promise<EngineState>;
  restore(document: Uint8Array): Promise<void>;
  undo(): Promise<boolean>;
  redo(): Promise<boolean>;
}

type WorkerMethods = Omit<EditorEngine, 'subscribe' | 'undo' | 'redo'> & WorkerOnlyMethods;

export type EngineMethod = keyof WorkerMethods;
export type EngineArgs<M extends EngineMethod> = Parameters<WorkerMethods[M]>;
export type EngineValue<M extends EngineMethod> = Awaited<ReturnType<WorkerMethods[M]>>;

export type EngineRequest = {
  [M in EngineMethod]: { readonly id: number; readonly method: M; readonly args: EngineArgs<M> };
}[EngineMethod];

export type EngineResponse =
  | { readonly id: number; readonly ok: true; readonly value: unknown }
  | { readonly id: number; readonly ok: false; readonly error: EngineError };

export interface StatePush {
  readonly type: 'state';
  readonly state: EngineState;
}

export type WorkerMessage = EngineResponse | StatePush;

/** The methods that change the engine: the worker pushes the state after each one. */
export const COMMANDS: ReadonlySet<EngineMethod> = new Set<EngineMethod>([
  'create',
  'open',
  'restore',
  'apply',
  'undo',
  'redo',
  'markSaved',
]);

/**
 * The buffers of the byte arrays in `value`, each once: posted as transferables, they move to
 * the other side instead of being copied, and are detached on this one.
 */
export function transferables(value: unknown): ArrayBuffer[] {
  const buffers = new Set<ArrayBuffer>();
  collectBuffers(value, buffers);
  return [...buffers];
}

function collectBuffers(value: unknown, buffers: Set<ArrayBuffer>): void {
  if (ArrayBuffer.isView(value)) {
    if (value.buffer instanceof ArrayBuffer) buffers.add(value.buffer);
  } else if (typeof value === 'object' && value !== null) {
    for (const item of Object.values(value)) collectBuffers(item, buffers);
  }
}
