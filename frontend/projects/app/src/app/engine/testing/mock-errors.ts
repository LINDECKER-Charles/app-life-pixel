import type { EngineError } from '../engine-types';

/** An `EngineError` the mock can `throw` (and so `Promise` can reject with). */
export class MockEngineError extends Error implements EngineError {
  readonly code: string;
  readonly params: Readonly<Record<string, unknown>>;

  constructor(code: string, params: Readonly<Record<string, unknown>> = {}) {
    super(code);
    this.code = code;
    this.params = params;
  }
}

export function layerNotFound(layer: number): MockEngineError {
  return new MockEngineError('edit.layer_not_found', { layer });
}

export function frameNotFound(frame: number): MockEngineError {
  return new MockEngineError('edit.frame_not_found', { frame });
}

export function tagNotFound(name: string): MockEngineError {
  return new MockEngineError('edit.tag_not_found', { name });
}

export function lastLayer(): MockEngineError {
  return new MockEngineError('edit.last_layer');
}

export function lastFrame(): MockEngineError {
  return new MockEngineError('edit.last_frame');
}

export function paletteFull(max: number): MockEngineError {
  return new MockEngineError('edit.palette_full', { max });
}

export function paletteIndex(max: number): MockEngineError {
  return new MockEngineError('document.palette', { max });
}

export function duplicateTag(name: string): MockEngineError {
  return new MockEngineError('document.tag', { name });
}
