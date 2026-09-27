import type { Limits } from '../../engine/engine-types';

/** What the new-animation dialog asks; the layer's name comes from the catalogue. */
export interface NewAnimationValues {
  readonly title: string;
  readonly width: number;
  readonly height: number;
}

/** The sides a canvas may measure, from the engine's limits. */
export interface SideBounds {
  readonly min: number;
  readonly max: number;
}

/** A new animation measures 32 × 32 unless the person asks otherwise. */
const PREFERRED_SIDE = 32;

export function sideBounds(limits: Limits): SideBounds {
  return { min: limits.canvasMinSide, max: limits.canvasMaxSide };
}

/** The default side, within the bounds. */
export function defaultSide({ min, max }: SideBounds): number {
  return Math.min(Math.max(PREFERRED_SIDE, min), max);
}

export function isValidSide(side: number, { min, max }: SideBounds): boolean {
  return Number.isInteger(side) && side >= min && side <= max;
}

/** A title holds 1 to `nameMaxChars` characters once trimmed. */
export function isValidTitle(title: string, limits: Limits): boolean {
  const length = [...title.trim()].length;
  return length > 0 && length <= limits.nameMaxChars;
}
