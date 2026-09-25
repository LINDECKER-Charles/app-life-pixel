import type { Limits } from '../../engine/engine-types';

export interface SpriteSheetValues {
  readonly cellWidth: number;
  readonly cellHeight: number;
  readonly durationMs: number;
}

export interface Bounds {
  readonly min: number;
  readonly max: number;
}

const MIN_CELL_SIDE = 1;

export function cellSideBounds(limits: Limits): Bounds {
  return { min: MIN_CELL_SIDE, max: limits.importMaxSide };
}

export function durationBounds(limits: Limits): Bounds {
  return { min: limits.minFrameDurationMs, max: limits.maxFrameDurationMs };
}

export function isValidSide(value: number, { min, max }: Bounds): boolean {
  return Number.isInteger(value) && value >= min && value <= max;
}
