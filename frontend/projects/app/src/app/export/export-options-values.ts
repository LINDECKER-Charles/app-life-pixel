import type { Limits } from '../engine/engine-types';

/** The scale factors a raster export may use, from the engine's limits. */
export interface ScaleBounds {
  readonly min: number;
  readonly max: number;
}

export function scaleBounds(limits: Limits): ScaleBounds {
  return { min: limits.exportMinScale, max: limits.exportMaxScale };
}

/** Keeps a typed scale an integer within the bounds; an invalid number falls back to the lowest. */
export function clampScale(scale: number, { min, max }: ScaleBounds): number {
  if (!Number.isFinite(scale)) return min;
  return Math.min(Math.max(Math.round(scale), min), max);
}
