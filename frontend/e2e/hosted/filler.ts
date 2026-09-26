/** The filler's canvas: the largest side, so that one cel holds enough runs. */
const SIDE = 512;
const PIXELS = SIDE * SIDE;

/** A run of `length` pixels of palette `index`: LEB128 length, then the index byte. */
function run(length: number, index: number): number[] {
  const bytes: number[] = [];
  let rest = length;
  do {
    const low = rest & 0x7f;
    rest >>>= 7;
    bytes.push(rest > 0 ? low | 0x80 : low);
  } while (rest > 0);
  return [...bytes, index];
}

/** A document whose cel alternates black and white on its first `runs` pixels. */
function document(title: string, runs: number): string {
  const rle: number[] = [];
  for (let pixel = 0; pixel < runs; pixel++) rle.push(1, 1 + (pixel % 2));
  if (runs < PIXELS) rle.push(...run(PIXELS - runs, 0));
  const animation = {
    format: 'life-pixel/animation',
    version: 1,
    title,
    width: SIDE,
    height: SIDE,
    palette: ['#00000000', '#000000ff', '#ffffffff'],
    layers: [{ id: 1, name: 'Layer 1', visible: true }],
    frames: [{ id: 2, durationMs: 100 }],
    cels: [{ layer: 1, frame: 2, rle: Buffer.from(rle).toString('base64') }],
    tags: [],
    nextId: 3,
  };
  return JSON.stringify(animation, null, 2);
}

/**
 * A valid animation document titled `title` of at most `maxBytes` bytes, and within a few bytes
 * of it: its size is chosen through the number of single-pixel runs of its one cel, each of which
 * adds 2 bytes before base64 (core.md, "Serialization").
 */
export function fillerDocument(title: string, maxBytes: number): string {
  const empty = Buffer.byteLength(document(title, 0));
  if (empty > maxBytes) throw new Error(`A filler takes at least ${empty} bytes.`);
  let runs = Math.min(PIXELS, Math.floor(((maxBytes - empty) * 3) / 8));
  let filler = document(title, runs);
  while (Buffer.byteLength(filler) > maxBytes) filler = document(title, --runs);
  return filler;
}
