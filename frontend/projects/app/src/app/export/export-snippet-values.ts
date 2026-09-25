import type { ExportedFile } from '../engine/engine-types';
import type { ExportRow } from './export-row';

const LOADER_FILE_NAME = 'life-pixel.js';
const WASM_MEDIA_TYPE = 'application/wasm';

function fileFrom(
  row: ExportRow | null,
  predicate: (file: ExportedFile) => boolean,
): ExportedFile | null {
  return row?.files.find(predicate) ?? null;
}

/** `/assets/<stem>.wasm`, the file the WASM row just downloaded, once it is ready. */
export function defaultSrc(wasmRow: ExportRow | null): string {
  const file = fileFrom(wasmRow, (candidate) => candidate.mediaType === WASM_MEDIA_TYPE);
  return file ? `/assets/${file.name}` : '';
}

/** `/assets/life-pixel.js`, the loader every animation of a site shares. */
export function defaultLoader(wasmRow: ExportRow | null): string {
  const file = fileFrom(wasmRow, (candidate) => candidate.name === LOADER_FILE_NAME);
  return `/assets/${file?.name ?? LOADER_FILE_NAME}`;
}
