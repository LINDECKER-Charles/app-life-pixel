/**
 * The animation both journeys draw — once with the pointer, once with the keyboard alone — and
 * what the exported player must show of it. Colours are indices of core's default palette.
 */

export interface Pixel {
  readonly x: number;
  readonly y: number;
}

export type Rgba = readonly [number, number, number, number];

/** The new animation's size: at the default zoom of 8 it fits any desktop viewport. */
export const SIZE = 16;
export const TITLE = 'Journey';

/** Palette indices of core's DEFAULT_PALETTE, and the colours the player draws for them. */
export const COLOR = {
  black: { index: 1, rgba: [0x00, 0x00, 0x00, 0xff] },
  red: { index: 6, rgba: [0xed, 0x1c, 0x24, 0xff] },
  green: { index: 9, rgba: [0x22, 0xb1, 0x4c, 0xff] },
  blue: { index: 10, rgba: [0x00, 0xa2, 0xe8, 0xff] },
} as const satisfies Record<string, { index: number; rgba: Rgba }>;

export const TRANSPARENT: Rgba = [0, 0, 0, 0];

/** Frame 1: a black pencil stroke, a red line, then a blue fill of the transparent rest. */
export const PENCIL_STROKE: readonly Pixel[] = [
  { x: 1, y: 1 },
  { x: 2, y: 1 },
  { x: 3, y: 1 },
];
export const LINE = { from: { x: 1, y: 4 }, to: { x: 6, y: 4 } } as const;
export const FILL_AT: Pixel = { x: 10, y: 10 };
/** Frame 2: one green pencil dot on an empty frame. */
export const DOT: Pixel = { x: 8, y: 8 };

/** Frame 1 is `idle`, looping; frame 2 is `blink`, played once. */
export const DURATIONS_MS = [200, 300] as const;
export const IDLE_TAG = 'idle';
export const BLINK_TAG = 'blink';

/** What the player must show, pixel by pixel, for each frame. */
export const EXPECTED_FRAMES: readonly (readonly { pixel: Pixel; rgba: Rgba }[])[] = [
  [
    { pixel: { x: 1, y: 1 }, rgba: COLOR.black.rgba },
    { pixel: { x: 3, y: 1 }, rgba: COLOR.black.rgba },
    { pixel: { x: 1, y: 4 }, rgba: COLOR.red.rgba },
    { pixel: { x: 6, y: 4 }, rgba: COLOR.red.rgba },
    { pixel: { x: 0, y: 0 }, rgba: COLOR.blue.rgba },
    { pixel: FILL_AT, rgba: COLOR.blue.rgba },
    { pixel: { x: 15, y: 15 }, rgba: COLOR.blue.rgba },
  ],
  [
    { pixel: DOT, rgba: COLOR.green.rgba },
    { pixel: { x: 0, y: 0 }, rgba: TRANSPARENT },
    { pixel: { x: 2, y: 1 }, rgba: TRANSPARENT },
    { pixel: { x: 3, y: 4 }, rgba: TRANSPARENT },
  ],
];
