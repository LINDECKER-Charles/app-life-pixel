import type { EditorEngine } from '../../engine/editor-engine';
import type { EditOperation, FrameId, RenderedFrame } from '../../engine/engine-types';
import type { OnionSkinFrame } from './canvas-renderer';

/** A document's frames, reduced to what onion-skin neighbour lookup needs. */
export type FrameList = readonly { readonly id: FrameId }[];

export interface OnionSkinSettings {
  readonly enabled: boolean;
  readonly before: number;
  readonly after: number;
}

export interface OnionSkinRequest {
  readonly frames: FrameList;
  readonly activeFrame: FrameId | null;
  readonly onionSkin: OnionSkinSettings;
}

/** The active frame's pixels, with the in-progress gesture's preview if there is one. */
export async function loadActiveFrame(
  engine: EditorEngine,
  frame: FrameId | null,
  preview: EditOperation | null,
): Promise<RenderedFrame | null> {
  if (frame === null) return null;
  return engine.render(preview ? { frame, preview } : { frame });
}

async function renderNeighbors(params: {
  readonly engine: EditorEngine;
  readonly frames: FrameList;
  readonly fromIndex: number;
  readonly direction: 1 | -1;
  readonly count: number;
}): Promise<OnionSkinFrame[]> {
  const { engine, frames, fromIndex, direction, count } = params;
  const skins: OnionSkinFrame[] = [];
  for (let step = 1; step <= count; step++) {
    const neighbor = frames[fromIndex + direction * step];
    if (!neighbor) break;
    skins.push({ image: await engine.render({ frame: neighbor.id }), stepsAway: step });
  }
  return skins;
}

/** The frames before and after the active one, rendered without a preview (editor.md, U2). */
export async function loadOnionSkin(
  engine: EditorEngine,
  request: OnionSkinRequest,
): Promise<{
  readonly before: readonly OnionSkinFrame[];
  readonly after: readonly OnionSkinFrame[];
}> {
  const { frames, activeFrame, onionSkin } = request;
  if (!onionSkin.enabled || activeFrame === null) return { before: [], after: [] };
  const fromIndex = frames.findIndex((candidate) => candidate.id === activeFrame);
  const [before, after] = await Promise.all([
    renderNeighbors({ engine, frames, fromIndex, direction: -1, count: onionSkin.before }),
    renderNeighbors({ engine, frames, fromIndex, direction: 1, count: onionSkin.after }),
  ]);
  return { before, after };
}
