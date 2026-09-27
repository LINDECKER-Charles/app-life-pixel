import type { Color, FakeAnimation } from './fake-export';

export const RED: Color = [255, 0, 0, 255];
export const GREEN: Color = [0, 255, 0, 255];
export const BLUE: Color = [0, 0, 255, 255];
export const WHITE: Color = [255, 255, 255, 255];

const FRAME_MS = 100;

/**
 * Four frames of 100 ms: `idle` loops over the first two, `jump` plays the last two once. Tag
 * `idle` is the default range.
 */
export const MASCOT: FakeAnimation = {
  width: 8,
  height: 6,
  title: 'The mascot',
  frames: [RED, GREEN, BLUE, WHITE].map((color) => ({ duration: FRAME_MS, color })),
  tags: [
    { name: 'idle', first: 0, last: 1 },
    { name: 'jump', first: 2, last: 3, isOnce: true },
  ],
};

/** Two frames of 100 ms without tags: the whole animation loops. */
export const BLINK: FakeAnimation = {
  frames: [WHITE, BLUE].map((color) => ({ duration: FRAME_MS, color })),
};
