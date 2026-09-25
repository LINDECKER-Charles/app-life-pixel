import type { Color } from '../engine/engine-types';

/** A palette colour, freshly added, opaque black. */
export const DEFAULT_COLOR: Color = '#000000ff';

const HEX8_PATTERN = /^#[0-9a-f]{8}$/i;

export interface Rgba {
  /** `#rrggbb`, for the `<input type="color">`. */
  readonly rgb: string;
  /** 0 to 255, for the alpha slider. */
  readonly alpha: number;
}

export function isValidHex8(value: string): boolean {
  return HEX8_PATTERN.test(value);
}

/** Splits a valid `#rrggbbaa` colour; an invalid one falls back to opaque black. */
export function splitHex8(hex: string): Rgba {
  if (!isValidHex8(hex)) return { rgb: '#000000', alpha: 255 };
  return { rgb: hex.slice(0, 7).toLowerCase(), alpha: Number.parseInt(hex.slice(7, 9), 16) };
}

/** Joins a `#rrggbb` colour and an alpha (0 to 255, clamped) into `#rrggbbaa`. */
export function joinHex8(rgb: string, alpha: number): Color {
  const clamped = Math.max(0, Math.min(255, Math.round(alpha)));
  return `${rgb.toLowerCase()}${clamped.toString(16).padStart(2, '0')}`;
}
