import type { DocumentSummary, EditOperation } from '../engine-types';
import {
  duplicateTag,
  frameNotFound,
  lastFrame,
  lastLayer,
  layerNotFound,
  paletteFull,
  paletteIndex,
  tagNotFound,
} from './mock-errors';
import {
  findFrameIndex,
  findLayerIndex,
  findTagIndex,
  shiftTagsForDelete,
  shiftTagsForInsert,
  strokeKey,
  type MockDocument,
} from './mock-state';

function requireLayer(summary: DocumentSummary, layer: number): number {
  const index = findLayerIndex(summary, layer);
  if (index < 0) throw layerNotFound(layer);
  return index;
}

function requireFrame(summary: DocumentSummary, frame: number): number {
  const index = findFrameIndex(summary, frame);
  if (index < 0) throw frameNotFound(frame);
  return index;
}

function requireTag(summary: DocumentSummary, name: string): number {
  const index = findTagIndex(summary, name);
  if (index < 0) throw tagNotFound(name);
  return index;
}

function clampPosition(position: number, length: number): number {
  return Math.min(Math.max(position, 0), length);
}

function withSummary(document: MockDocument, summary: DocumentSummary): MockDocument {
  return { ...document, summary };
}

/** The pencil, or the eraser with index 0: the only pixel content the mock keeps. */
function applyPaintStroke(document: MockDocument, operation: EditOperation): MockDocument {
  const stroke = operation as Extract<EditOperation, { kind: 'paintStroke' }>;
  requireLayer(document.summary, stroke.layer);
  requireFrame(document.summary, stroke.frame);
  const strokes = new Map(document.strokes);
  strokes.set(strokeKey(stroke.layer, stroke.frame), {
    index: stroke.index,
    points: stroke.points,
  });
  return { ...document, strokes };
}

/** `fill`, `line`, `rectangle`, `moveSelection`, `importImage`: recorded without pixels. */
function applyPixelOperation(document: MockDocument, operation: EditOperation): MockDocument {
  const pixels = operation as Extract<EditOperation, { kind: 'fill' }>;
  requireLayer(document.summary, pixels.layer);
  requireFrame(document.summary, pixels.frame);
  return document;
}

function applySetPaletteEntry(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'setPaletteEntry' }>;
  const { palette } = document.summary;
  if (change.index < 1 || change.index >= palette.length) throw paletteIndex(palette.length - 1);
  const next = palette.map((entry, index) => (index === change.index ? change.color : entry));
  return withSummary(document, { ...document.summary, palette: next });
}

function applyAddPaletteEntry(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'addPaletteEntry' }>;
  const { palette } = document.summary;
  if (palette.length >= 256) throw paletteFull(256);
  return withSummary(document, { ...document.summary, palette: [...palette, change.color] });
}

function applyRemovePaletteEntry(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'removePaletteEntry' }>;
  const { palette } = document.summary;
  if (change.index < 1 || change.index >= palette.length) throw paletteIndex(palette.length - 1);
  const next = palette.filter((_, index) => index !== change.index);
  return withSummary(document, { ...document.summary, palette: next });
}

function applyMovePaletteEntry(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'movePaletteEntry' }>;
  const { palette } = document.summary;
  const bounds =
    change.from < 1 ||
    change.from >= palette.length ||
    change.to < 1 ||
    change.to >= palette.length;
  if (bounds) throw paletteIndex(palette.length - 1);
  const next = [...palette];
  next.splice(change.to, 0, ...next.splice(change.from, 1));
  return withSummary(document, { ...document.summary, palette: next });
}

function applyAddLayer(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'addLayer' }>;
  const { layers } = document.summary;
  const layer = { id: document.nextId, name: change.name, visible: true };
  const next = [...layers];
  next.splice(clampPosition(change.position, layers.length), 0, layer);
  return {
    ...document,
    nextId: document.nextId + 1,
    summary: { ...document.summary, layers: next },
  };
}

function applyDeleteLayer(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'deleteLayer' }>;
  requireLayer(document.summary, change.layer);
  const { layers } = document.summary;
  if (layers.length === 1) throw lastLayer();
  const next = layers.filter((layer) => layer.id !== change.layer);
  return withSummary(document, { ...document.summary, layers: next });
}

function applyMoveLayer(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'moveLayer' }>;
  const from = requireLayer(document.summary, change.layer);
  const layers = [...document.summary.layers];
  layers.splice(clampPosition(change.position, layers.length), 0, ...layers.splice(from, 1));
  return withSummary(document, { ...document.summary, layers });
}

function applyRenameLayer(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'renameLayer' }>;
  requireLayer(document.summary, change.layer);
  const layers = document.summary.layers.map((layer) =>
    layer.id === change.layer ? { ...layer, name: change.name } : layer,
  );
  return withSummary(document, { ...document.summary, layers });
}

function applySetLayerVisibility(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'setLayerVisibility' }>;
  requireLayer(document.summary, change.layer);
  const layers = document.summary.layers.map((layer) =>
    layer.id === change.layer ? { ...layer, visible: change.visible } : layer,
  );
  return withSummary(document, { ...document.summary, layers });
}

function applyAddFrame(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'addFrame' }>;
  const { frames } = document.summary;
  const position = clampPosition(change.position, frames.length);
  const next = [...frames];
  next.splice(position, 0, { id: document.nextId, durationMs: change.durationMs });
  const tags = shiftTagsForInsert(document.summary.tags, position);
  return {
    ...document,
    nextId: document.nextId + 1,
    summary: { ...document.summary, frames: next, tags },
  };
}

function applyDuplicateFrame(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'duplicateFrame' }>;
  const at = requireFrame(document.summary, change.frame);
  const frames = [...document.summary.frames];
  frames.splice(at + 1, 0, { ...frames[at], id: document.nextId });
  const tags = shiftTagsForInsert(document.summary.tags, at + 1);
  return {
    ...document,
    nextId: document.nextId + 1,
    summary: { ...document.summary, frames, tags },
  };
}

function applyDeleteFrame(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'deleteFrame' }>;
  const at = requireFrame(document.summary, change.frame);
  const { frames } = document.summary;
  if (frames.length === 1) throw lastFrame();
  const next = frames.filter((frame) => frame.id !== change.frame);
  const tags = shiftTagsForDelete(document.summary.tags, at);
  return withSummary(document, { ...document.summary, frames: next, tags });
}

/** Moving a frame keeps every tag's positions unchanged (core.md, K2). */
function applyMoveFrame(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'moveFrame' }>;
  const from = requireFrame(document.summary, change.frame);
  const frames = [...document.summary.frames];
  frames.splice(clampPosition(change.position, frames.length), 0, ...frames.splice(from, 1));
  return withSummary(document, { ...document.summary, frames });
}

function applySetFrameDuration(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'setFrameDuration' }>;
  requireFrame(document.summary, change.frame);
  const frames = document.summary.frames.map((frame) =>
    frame.id === change.frame ? { ...frame, durationMs: change.durationMs } : frame,
  );
  return withSummary(document, { ...document.summary, frames });
}

function applyAddTag(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'addTag' }>;
  if (findTagIndex(document.summary, change.tag.name) >= 0) throw duplicateTag(change.tag.name);
  return withSummary(document, {
    ...document.summary,
    tags: [...document.summary.tags, change.tag],
  });
}

function applyUpdateTag(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'updateTag' }>;
  const at = requireTag(document.summary, change.name);
  const tags = document.summary.tags.map((tag, index) => (index === at ? change.tag : tag));
  return withSummary(document, { ...document.summary, tags });
}

function applyDeleteTag(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'deleteTag' }>;
  requireTag(document.summary, change.name);
  const tags = document.summary.tags.filter((tag) => tag.name !== change.name);
  return withSummary(document, { ...document.summary, tags });
}

function applyReplaceTags(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'replaceTags' }>;
  return withSummary(document, { ...document.summary, tags: change.tags });
}

function applySetTitle(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'setTitle' }>;
  return withSummary(document, { ...document.summary, title: change.title });
}

/**
 * The mock does not decode PNG bytes (that is `editor-wasm`'s job, W1): the import is validated
 * and recorded, with no structural or pixel change. A departure from core.md's exact behaviour,
 * reported in W0's decisions.
 */
function applyImportSpriteSheet(document: MockDocument, operation: EditOperation): MockDocument {
  const change = operation as Extract<EditOperation, { kind: 'importSpriteSheet' }>;
  requireLayer(document.summary, change.layer);
  return document;
}

const handlers: Record<
  EditOperation['kind'],
  (document: MockDocument, operation: EditOperation) => MockDocument
> = {
  paintStroke: applyPaintStroke,
  fill: applyPixelOperation,
  line: applyPixelOperation,
  rectangle: applyPixelOperation,
  moveSelection: applyPixelOperation,
  importImage: applyPixelOperation,
  setPaletteEntry: applySetPaletteEntry,
  addPaletteEntry: applyAddPaletteEntry,
  removePaletteEntry: applyRemovePaletteEntry,
  movePaletteEntry: applyMovePaletteEntry,
  addLayer: applyAddLayer,
  deleteLayer: applyDeleteLayer,
  moveLayer: applyMoveLayer,
  renameLayer: applyRenameLayer,
  setLayerVisibility: applySetLayerVisibility,
  addFrame: applyAddFrame,
  duplicateFrame: applyDuplicateFrame,
  deleteFrame: applyDeleteFrame,
  moveFrame: applyMoveFrame,
  setFrameDuration: applySetFrameDuration,
  addTag: applyAddTag,
  updateTag: applyUpdateTag,
  deleteTag: applyDeleteTag,
  replaceTags: applyReplaceTags,
  setTitle: applySetTitle,
  importSpriteSheet: applyImportSpriteSheet,
};

export function applyOperation(document: MockDocument, operation: EditOperation): MockDocument {
  return handlers[operation.kind](document, operation);
}
