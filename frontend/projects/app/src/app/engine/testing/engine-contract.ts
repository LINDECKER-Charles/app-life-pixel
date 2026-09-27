import { describe, expect, it } from 'vitest';
import type { EditorEngine } from '../editor-engine';
import type { EngineState, ExportFormat, NewAnimationOptions } from '../engine-types';

const NEW_ANIMATION: NewAnimationOptions = {
  title: 'Contract',
  width: 4,
  height: 4,
  layerName: 'Base',
};
const EXPORT_FORMATS: readonly ExportFormat[] = [
  'wasm',
  'gif',
  'apng',
  'sprite_sheet',
  'png_frames',
];

function captureState(engine: EditorEngine): () => EngineState {
  let latest: EngineState | undefined;
  engine.subscribe((state) => {
    latest = state;
  });
  return () => {
    if (!latest) throw new Error('the engine published no state yet');
    return latest;
  };
}

/**
 * The behaviour any `EditorEngine` must have, checked here against the mock and, from W1 on,
 * against `WasmEditorEngine` in a real browser (editor.md, W0 and W1).
 */
export function runEngineContract(create: () => Promise<EditorEngine>): void {
  describe('the engine contract', () => {
    it('gives a ready state with the document once created', async () => {
      const engine = await create();
      const state = captureState(engine);
      await engine.create(NEW_ANIMATION);
      expect(state().status).toBe('ready');
      expect(state().document?.title).toBe(NEW_ANIMATION.title);
      expect(state().document?.layers).toHaveLength(1);
    });

    it('changes the summary on a structural operation', async () => {
      const engine = await create();
      const state = captureState(engine);
      await engine.create(NEW_ANIMATION);
      await engine.apply({ kind: 'addLayer', position: 0, name: 'Detail' });
      expect(state().document?.layers).toHaveLength(2);
    });

    it('toggles the summary and the undo and redo flags', async () => {
      const engine = await create();
      const state = captureState(engine);
      await engine.create(NEW_ANIMATION);
      await engine.apply({ kind: 'addLayer', position: 0, name: 'Detail' });
      expect(state().canUndo).toBe(true);
      expect(state().canRedo).toBe(false);

      await engine.undo();
      expect(state().document?.layers).toHaveLength(1);
      expect(state().canUndo).toBe(false);
      expect(state().canRedo).toBe(true);

      await engine.redo();
      expect(state().document?.layers).toHaveLength(2);
      expect(state().canRedo).toBe(false);
    });

    it('has unsaved work after a change, and none after markSaved', async () => {
      const engine = await create();
      const state = captureState(engine);
      await engine.create(NEW_ANIMATION);
      expect(state().hasUnsavedWork).toBe(false);

      await engine.apply({ kind: 'setTitle', title: 'Renamed' });
      expect(state().hasUnsavedWork).toBe(true);

      await engine.markSaved();
      expect(state().hasUnsavedWork).toBe(false);
    });

    it('opens what it serialized, unchanged', async () => {
      const engine = await create();
      const state = captureState(engine);
      await engine.create(NEW_ANIMATION);
      await engine.apply({ kind: 'setTitle', title: 'Roundtrip' });
      const before = state().document;

      await engine.open(await engine.serialize());
      expect(state().document).toEqual(before);
    });

    it('leaves the state unchanged when an operation is rejected', async () => {
      const engine = await create();
      const state = captureState(engine);
      await engine.create(NEW_ANIMATION);
      const before = state();

      await expect(engine.apply({ kind: 'deleteLayer', layer: 9_999 })).rejects.toMatchObject({
        code: 'edit.layer_not_found',
      });
      expect(state()).toEqual(before);
    });

    it('answers with files for every export format', async () => {
      const engine = await create();
      await engine.create(NEW_ANIMATION);
      for (const format of EXPORT_FORMATS) {
        const result = await engine.export({ format });
        expect(result.files.length).toBeGreaterThan(0);
      }
    });
  });
}
