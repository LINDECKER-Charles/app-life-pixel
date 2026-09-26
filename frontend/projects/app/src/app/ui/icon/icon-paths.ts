/**
 * An icon's drawing on the 24-unit grid: the outline, stroked at 1.75 with round caps and joins,
 * and an optional solid part, filled as well. A dot is a zero-length segment (`h.01`), which the
 * round cap draws as a disc.
 */
export interface IconShape {
  readonly outline: string;
  readonly solid?: string;
}

// Shapes more than one icon shares.
const CIRCLE = 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z';
const LENS = 'M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4-4';
const TWO_SHEETS =
  'M11 9h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2z' +
  'M15 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h3';
const EYE = 'M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z';
const RECTANGLE = 'M4 6h16v12H4z';

/**
 * Every icon of the app, one outline family after design-system/preview/sections/studio.html:
 * the drawing tools, history, imports, view, frames and layers, playback, files, feedback and
 * the shell. Add a name here before using it; `lp-icon` accepts no other.
 */
export const ICON_PATHS = {
  pencil: { outline: 'm5 15 11-11 4 4L9 19l-6 2zM14 6l4 4M5 15l4 4' },
  eraser: { outline: 'm3 14 11-11 7 7-11 11H8zM8 9l7 7M10 21h11' },
  fill: { outline: 'm4 11 8-8 8 8-8 8zM4 11h16M8 2l4 5M20 16l2 4h-4z' },
  line: {
    outline: 'M7.5 16.5l9-9M4 18a2 2 0 1 0 4 0 2 2 0 1 0-4 0M16 6a2 2 0 1 0 4 0 2 2 0 1 0-4 0',
  },
  rectangle: { outline: RECTANGLE },
  'rectangle-filled': { outline: RECTANGLE, solid: RECTANGLE },
  select: { outline: 'M4 7V4h3M10 4h4M17 4h3v3M20 10v4M20 17v3h-3M14 20h-4M7 20H4v-3M4 14v-4' },
  undo: { outline: 'M9 14 4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 0 11H11' },
  redo: { outline: 'm15 14 5-5-5-5M20 9H9.5a5.5 5.5 0 0 0 0 11H13' },
  'import-image': {
    outline:
      'M13 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7M4 16l4-4 3 3 2-2 5 5M18 2v6M15 5l3 3 3-3',
  },
  'import-sheet': { outline: 'M4 4h7v7H4zM4 13h7v7H4zM13 13h7v7h-7zM16.5 3v7M13.5 7l3 3 3-3' },
  help: { outline: `${CIRCLE}M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .9-1 1.6v.6M12 17h.01` },
  'zoom-in': { outline: `${LENS}M8 11h6M11 8v6` },
  'zoom-out': { outline: `${LENS}M8 11h6` },
  fit: { outline: 'M4 9V4h5M15 4h5v5M20 15v5h-5M9 20H4v-5' },
  grid: { outline: 'M4 4h16v16H4zM4 9.33h16M4 14.67h16M9.33 4v16M14.67 4v16' },
  'onion-skin': { outline: 'M9 9h11v11H9zM4 12v3h2M4 8V4h4M12 4h3v2' },
  eye: { outline: EYE },
  'eye-off': { outline: `${EYE}M4 4l16 16` },
  trash: {
    outline: 'M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3',
  },
  plus: { outline: 'M12 5v14M5 12h14' },
  duplicate: { outline: `${TWO_SHEETS}M14.5 12v5M12 14.5h5` },
  edit: {
    outline:
      'M12 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-6' +
      'M17.5 3.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4z',
  },
  play: { outline: 'M7 4v16l13-8z' },
  pause: { outline: 'M8 5v14M16 5v14' },
  stop: { outline: 'M6 6h12v12H6z' },
  save: {
    outline: 'M5 3h11l5 5v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM7 3v5h8V3M7 21v-7h10v7',
  },
  export: { outline: 'M12 15V3M7 8l5-5 5 5M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6' },
  download: { outline: 'M12 3v12M7 10l5 5 5-5M5 20h14' },
  copy: { outline: TWO_SHEETS },
  check: { outline: 'M4 12.5l5 5L20 6.5' },
  close: { outline: 'M6 6l12 12M18 6 6 18' },
  'chevron-down': { outline: 'm6 9 6 6 6-6' },
  'chevron-up': { outline: 'm6 15 6-6 6 6' },
  'chevron-left': { outline: 'm15 6-6 6 6 6' },
  'chevron-right': { outline: 'm9 6 6 6-6 6' },
  menu: { outline: 'M4 6h16M4 12h16M4 18h16' },
  warning: { outline: 'M12 3 2 20h20zM12 9v5M12 17h.01' },
  error: { outline: 'M8 3h8l5 5v8l-5 5H8l-5-5V8zM12 7.5v5.5M12 16.5h.01' },
  info: { outline: `${CIRCLE}M12 11v5.5M12 7.5h.01` },
  success: { outline: `${CIRCLE}M8 12.5l2.5 2.5L16 9.5` },
  layer: { outline: 'M12 3 3 7.5l9 4.5 9-4.5zM3 12l9 4.5 9-4.5M3 16.5 12 21l9-4.5' },
  image: {
    outline:
      'M5 4h14a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z' +
      'M4 16l5-5 4 4 2-2 5 5M15.5 8.5h.01',
  },
  frame: { outline: 'M5 3h14v18H5zM5 7h14M5 17h14M8 3v4M16 3v4M8 17v4M16 17v4' },
  tag: { outline: 'M3 4v7.6l9.4 9.4 8.6-8.6L11.6 3H4a1 1 0 0 0-1 1zM7.5 7.5h.01' },
  settings: { outline: 'M4 7h9M17 7h3M4 17h3M11 17h9M15 5v4M9 15v4' },
  account: { outline: 'M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 21a8 8 0 0 1 16 0' },
  library: { outline: 'M5 4h4v16H5zM10 4h4v16h-4zM15 5.2l3.8-1 3.7 14.6-3.8 1z' },
  search: { outline: LENS },
  'sign-out': { outline: 'M10 4H5a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1h5M15 8l4 4-4 4M19 12H9' },
} as const satisfies Record<string, IconShape>;

/** The name of an icon of `ICON_PATHS`. */
export type IconName = keyof typeof ICON_PATHS;
