import type { EditorEngine } from './editor-engine';
import { WasmEditorEngine } from './wasm/wasm-editor-engine';

/**
 * The engine `EDITOR_ENGINE` provides: editor-wasm in a Web Worker (editor.md, W1). The unit
 * tests' build replaces this file with testing/default-engine.ts (angular.json, `mock-engine`),
 * so that they run without Rust.
 */
export function createDefaultEngine(): EditorEngine {
  return new WasmEditorEngine();
}
