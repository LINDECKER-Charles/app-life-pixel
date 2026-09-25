import type { ExportedFile } from '../engine/engine-types';

async function gzipSize(bytes: Uint8Array): Promise<number> {
  const gzip = new CompressionStream('gzip');
  const writer = gzip.writable.getWriter();
  void writer.write(new Uint8Array(bytes)).then(() => writer.close());
  const compressed = await new Response(gzip.readable).arrayBuffer();
  return compressed.byteLength;
}

/** The combined gzip size of every file an export produced, measured with `CompressionStream`. */
export async function gzipTotal(files: readonly ExportedFile[]): Promise<number> {
  const sizes = await Promise.all(files.map((file) => gzipSize(file.bytes)));
  return sizes.reduce((total, size) => total + size, 0);
}

const UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte'] as const;

/** A byte count formatted with `Intl.NumberFormat`'s unit style, scaled to the nearest unit. */
export function formatBytes(bytes: number, locale: string): string {
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < UNITS.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return new Intl.NumberFormat(locale, {
    style: 'unit',
    unit: UNITS[unitIndex],
    unitDisplay: 'short',
    maximumFractionDigits: unitIndex === 0 ? 0 : 1,
  }).format(value);
}
