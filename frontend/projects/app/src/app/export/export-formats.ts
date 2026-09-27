import type { ExportFormat } from '../engine/engine-types';

/** The five export formats, in the order the dialog lists them (editor.md, U5). */
export const EXPORT_FORMATS: readonly ExportFormat[] = [
  'wasm',
  'gif',
  'apng',
  'sprite_sheet',
  'png_frames',
];

/** Formats a scale applies to; WASM embeds the document at its native resolution. */
const RASTER_FORMATS = new Set<ExportFormat>(['gif', 'apng', 'sprite_sheet', 'png_frames']);

export function isRasterFormat(format: ExportFormat): boolean {
  return RASTER_FORMATS.has(format);
}
