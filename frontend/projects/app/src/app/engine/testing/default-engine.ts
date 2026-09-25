import type { EditorEngine } from '../editor-engine';
import { MockEditorEngine } from './mock-editor-engine';

/** default-engine.ts in the unit tests' build (angular.json, `mock-engine`): the mock. */
export function createDefaultEngine(): EditorEngine {
  return new MockEditorEngine();
}
