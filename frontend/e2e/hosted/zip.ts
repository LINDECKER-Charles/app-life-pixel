import { inflateRawSync } from 'node:zlib';

const END_OF_DIRECTORY = 0x06054b50;
const DIRECTORY_ENTRY = 0x02014b50;
const LOCAL_HEADER = 0x04034b50;
const STORED = 0;
const DEFLATED = 8;

/** A file of a zip: its path, and its bytes on demand. */
export interface ZipEntry {
  readonly name: string;
  read(): Buffer;
}

function endOfDirectory(zip: Buffer): number {
  for (let offset = zip.length - 22; offset >= 0; offset--) {
    if (zip.readUInt32LE(offset) === END_OF_DIRECTORY) return offset;
  }
  throw new Error('Not a zip: no end of central directory.');
}

/** Where an entry's data starts — its local header —, how it is compressed, and its size. */
interface Stored {
  readonly header: number;
  readonly method: number;
  readonly size: number;
}

function contentOf(zip: Buffer, { header, method, size }: Stored): Buffer {
  if (zip.readUInt32LE(header) !== LOCAL_HEADER) throw new Error('A zip entry has no header.');
  const start = header + 30 + zip.readUInt16LE(header + 26) + zip.readUInt16LE(header + 28);
  const data = zip.subarray(start, start + size);
  if (method === STORED) return data;
  if (method === DEFLATED) return inflateRawSync(data);
  throw new Error(`Unsupported zip compression method ${method}.`);
}

/**
 * The entries of a zip, as its central directory lists them: enough to check what a data export
 * holds, stored or deflated, without a dependency.
 */
export function readZip(zip: Buffer): ZipEntry[] {
  const end = endOfDirectory(zip);
  const count = zip.readUInt16LE(end + 10);
  let offset = zip.readUInt32LE(end + 16);
  const entries: ZipEntry[] = [];
  for (let index = 0; index < count; index++) {
    if (zip.readUInt32LE(offset) !== DIRECTORY_ENTRY) throw new Error('A zip entry is damaged.');
    const method = zip.readUInt16LE(offset + 10);
    const size = zip.readUInt32LE(offset + 20);
    const nameLength = zip.readUInt16LE(offset + 28);
    const header = zip.readUInt32LE(offset + 42);
    const name = zip.toString('utf8', offset + 46, offset + 46 + nameLength);
    entries.push({ name, read: () => contentOf(zip, { header, method, size }) });
    offset += 46 + nameLength + zip.readUInt16LE(offset + 30) + zip.readUInt16LE(offset + 32);
  }
  return entries;
}
