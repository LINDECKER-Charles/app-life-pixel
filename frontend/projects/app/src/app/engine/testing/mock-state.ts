import type { Color, DocumentSummary, NewAnimationOptions, TagSpec } from '../engine-types';

/** core.md's `DEFAULT_PALETTE`, transcribed so the mock needs no engine to draw with. */
export const DEFAULT_PALETTE: readonly Color[] = [
  '#00000000',
  '#000000ff',
  '#ffffffff',
  '#7f7f7fff',
  '#c3c3c3ff',
  '#880015ff',
  '#ed1c24ff',
  '#ff7f27ff',
  '#fff200ff',
  '#22b14cff',
  '#00a2e8ff',
  '#3f48ccff',
  '#a349a4ff',
  '#b97a57ff',
  '#ffaec9ff',
  '#99d9eaff',
];

/**
 * The mock's whole world: a `DocumentSummary`, the ids not yet spent (mirroring core's shared
 * `next_id` for layers and frames), and the last stroke painted on each cel — the only pixel
 * content the mock keeps (editor.md, W0).
 */
export interface MockDocument {
  readonly summary: DocumentSummary;
  readonly nextId: number;
  readonly strokes: ReadonlyMap<
    string,
    {
      readonly index: number;
      readonly points: readonly { readonly x: number; readonly y: number }[];
    }
  >;
}

export function strokeKey(layer: number, frame: number): string {
  return `${layer}:${frame}`;
}

export function newDocument(options: NewAnimationOptions): MockDocument {
  const layerId = 1;
  const frameId = 2;
  const summary: DocumentSummary = {
    title: options.title,
    width: options.width,
    height: options.height,
    palette: options.palette ?? DEFAULT_PALETTE,
    layers: [{ id: layerId, name: options.layerName, visible: true }],
    frames: [{ id: frameId, durationMs: options.frameDurationMs ?? 100 }],
    tags: [],
  };
  return { summary, nextId: 3, strokes: new Map() };
}

/**
 * `serialize` and `open` round-trip the `DocumentSummary` alone: the mock keeps no real pixels
 * for the other operations, so there is nothing else to save (editor.md, W0).
 */
export function parseDocument(bytes: Uint8Array): MockDocument {
  const summary = JSON.parse(new TextDecoder().decode(bytes)) as DocumentSummary;
  const ids = [
    ...summary.layers.map((layer) => layer.id),
    ...summary.frames.map((frame) => frame.id),
  ];
  return { summary, nextId: Math.max(0, ...ids) + 1, strokes: new Map() };
}

export function serializeDocument(document: MockDocument): Uint8Array {
  return new TextEncoder().encode(JSON.stringify(document.summary));
}

export function findLayerIndex(summary: DocumentSummary, layer: number): number {
  return summary.layers.findIndex((candidate) => candidate.id === layer);
}

export function findFrameIndex(summary: DocumentSummary, frame: number): number {
  return summary.frames.findIndex((candidate) => candidate.id === frame);
}

export function findTagIndex(summary: DocumentSummary, name: string): number {
  return summary.tags.findIndex((candidate) => candidate.name === name);
}

/** Inserting a frame at `position` moves or extends every tag at or after it (core.md, K2). */
export function shiftTagsForInsert(tags: readonly TagSpec[], position: number): readonly TagSpec[] {
  return tags.map((tag) => ({
    ...tag,
    first: tag.first >= position ? tag.first + 1 : tag.first,
    last: tag.last >= position ? tag.last + 1 : tag.last,
  }));
}

/** Deleting a frame at `position` mirrors an insert, and drops a tag of that frame alone. */
export function shiftTagsForDelete(tags: readonly TagSpec[], position: number): readonly TagSpec[] {
  return tags
    .filter((tag) => !(tag.first === position && tag.last === position))
    .map((tag) => ({
      ...tag,
      first: tag.first > position ? tag.first - 1 : tag.first,
      last: tag.last > position ? tag.last - 1 : tag.last,
    }));
}
