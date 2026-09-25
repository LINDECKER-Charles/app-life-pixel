import type {
  DocumentSummary,
  ExportedFile,
  ExportRequest,
  ExportResult,
  SnippetRequest,
} from '../engine-types';

/** A simplified `compiler::file_stem`: enough for the mock's placeholder file names. */
export function fileStem(title: string): string {
  const slug = title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return slug.length > 0 ? slug : 'animation';
}

function encodeJson(value: unknown): Uint8Array {
  return new TextEncoder().encode(JSON.stringify(value));
}

function wasmFiles(payload: unknown, stem: string): readonly ExportedFile[] {
  return [
    { name: `${stem}.wasm`, mediaType: 'application/wasm', bytes: encodeJson(payload) },
    { name: 'life-pixel.js', mediaType: 'text/javascript', bytes: new TextEncoder().encode('') },
  ];
}

function placeholderFiles(
  summary: DocumentSummary,
  request: ExportRequest,
  stem: string,
): readonly ExportedFile[] {
  const payload = { summary, tag: request.tag ?? null, scale: request.scale ?? 1 };
  switch (request.format) {
    case 'wasm':
      return wasmFiles(payload, stem);
    case 'gif':
      return [{ name: `${stem}.gif`, mediaType: 'image/gif', bytes: encodeJson(payload) }];
    case 'apng':
      return [{ name: `${stem}.png`, mediaType: 'image/apng', bytes: encodeJson(payload) }];
    case 'sprite_sheet':
      return [
        { name: `${stem}.png`, mediaType: 'image/png', bytes: encodeJson(payload) },
        { name: `${stem}.json`, mediaType: 'application/json', bytes: encodeJson(payload) },
      ];
    case 'png_frames':
      return [
        { name: `${stem}-frames.zip`, mediaType: 'application/zip', bytes: encodeJson(payload) },
      ];
  }
}

/**
 * A placeholder export: real bytes come from `editor-wasm` and `compiler` (W1, C2). The mock only
 * needs plausible file names, media types and non-empty bytes for every format.
 */
export function buildExport(summary: DocumentSummary, request: ExportRequest): ExportResult {
  const files = placeholderFiles(summary, request, fileStem(summary.title));
  return { files, totalBytes: files.reduce((total, file) => total + file.bytes.byteLength, 0) };
}

/** A placeholder snippet: the real templates are `player-js`'s, rendered by `compiler` (W1). */
export function buildSnippet(request: SnippetRequest): string {
  const tag = request.tag ?? '';
  return [
    `<!-- ${request.framework} snippet -->`,
    `<life-pixel src="${request.src}" loader="${request.loader}" tag="${tag}" alt="${request.alt}"></life-pixel>`,
  ].join('\n');
}
