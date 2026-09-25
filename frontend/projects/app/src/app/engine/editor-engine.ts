import { InjectionToken } from '@angular/core';
import { createDefaultEngine } from './default-engine';
import type {
  EditOperation,
  EngineState,
  ExportRequest,
  ExportResult,
  NewAnimationOptions,
  RenderedFrame,
  RenderRequest,
  SnippetRequest,
} from './engine-types';

export interface EditorEngine {
  /* Commands: they resolve without a value, or reject with an EngineError. */
  create(options: NewAnimationOptions): Promise<void>;
  open(document: Uint8Array): Promise<void>;
  apply(operation: EditOperation): Promise<void>;
  undo(): Promise<void>;
  redo(): Promise<void>;
  markSaved(): Promise<void>;
  /* Queries. */
  render(request: RenderRequest): Promise<RenderedFrame>;
  serialize(): Promise<Uint8Array>;
  export(request: ExportRequest): Promise<ExportResult>;
  snippet(request: SnippetRequest): Promise<string>;
  /* State, published after every command; returns the unsubscribe function. */
  subscribe(listener: (state: EngineState) => void): () => void;
}

/**
 * The engine of the page: `WasmEditorEngine` in the app, the in-memory mock in the unit tests,
 * whose build swaps default-engine.ts for testing/default-engine.ts.
 */
export const EDITOR_ENGINE = new InjectionToken<EditorEngine>('EDITOR_ENGINE', {
  providedIn: 'root',
  factory: createDefaultEngine,
});
