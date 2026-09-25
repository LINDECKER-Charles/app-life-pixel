import type { EditorEngine } from '../editor-engine';
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
import { buildExport, buildSnippet } from './mock-export';
import {
  canRedo,
  canUndo,
  currentDocument,
  hasUnsavedWork,
  initialHistory,
  markHistorySaved,
  pushHistory,
  redoHistory,
  undoHistory,
  type MockHistory,
} from './mock-history';
import { MOCK_LIMITS } from './mock-limits';
import { applyOperation } from './mock-operations';
import { renderFrame } from './mock-render';
import { newDocument, parseDocument, serializeDocument } from './mock-state';

/**
 * The in-memory `EditorEngine` of editor.md's W0: keeps a `DocumentSummary` and applies the
 * structural operations to it, so the interface is built before `editor-wasm` exists (W1).
 */
export class MockEditorEngine implements EditorEngine {
  private history: MockHistory | null = null;
  private readonly listeners = new Set<(state: EngineState) => void>();

  async create(options: NewAnimationOptions): Promise<void> {
    this.history = initialHistory(newDocument(options));
    this.publish();
  }

  async open(document: Uint8Array): Promise<void> {
    this.history = initialHistory(parseDocument(document));
    this.publish();
  }

  async apply(operation: EditOperation): Promise<void> {
    const history = this.requireHistory();
    this.history = pushHistory(history, applyOperation(currentDocument(history), operation));
    this.publish();
  }

  async undo(): Promise<void> {
    if (this.history) this.history = undoHistory(this.history);
    this.publish();
  }

  async redo(): Promise<void> {
    if (this.history) this.history = redoHistory(this.history);
    this.publish();
  }

  async markSaved(): Promise<void> {
    if (this.history) this.history = markHistorySaved(this.history);
    this.publish();
  }

  async render(request: RenderRequest): Promise<RenderedFrame> {
    return renderFrame(currentDocument(this.requireHistory()), request.frame, request.preview);
  }

  async serialize(): Promise<Uint8Array> {
    return serializeDocument(currentDocument(this.requireHistory()));
  }

  async export(request: ExportRequest): Promise<ExportResult> {
    return buildExport(currentDocument(this.requireHistory()).summary, request);
  }

  async snippet(request: SnippetRequest): Promise<string> {
    return buildSnippet(request);
  }

  subscribe(listener: (state: EngineState) => void): () => void {
    this.listeners.add(listener);
    listener(this.computeState());
    return () => this.listeners.delete(listener);
  }

  private requireHistory(): MockHistory {
    if (!this.history)
      throw new Error('the mock engine has no open document: call create or open first');
    return this.history;
  }

  private computeState(): EngineState {
    if (!this.history) {
      return {
        status: 'empty',
        document: null,
        canUndo: false,
        canRedo: false,
        hasUnsavedWork: false,
        limits: MOCK_LIMITS,
      };
    }
    return {
      status: 'ready',
      document: currentDocument(this.history).summary,
      canUndo: canUndo(this.history),
      canRedo: canRedo(this.history),
      hasUnsavedWork: hasUnsavedWork(this.history),
      limits: MOCK_LIMITS,
    };
  }

  private publish(): void {
    const state = this.computeState();
    for (const listener of this.listeners) listener(state);
  }
}
