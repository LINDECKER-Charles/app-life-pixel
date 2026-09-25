import type { CompiledExport } from './module-cache';

/** The player ABI this loader plays. */
const ABI_VERSION = 1;
/** `set_tag`'s index for the whole animation, `0xFFFFFFFF` as an `i32`. */
const WHOLE_ANIMATION = -1;
/** The status `alloc` stands for when it returns `0`. */
const OUT_OF_MEMORY = 6;

/** `tick`'s bit 0: the framebuffer changed. */
export const FRAME_CHANGED = 1;
/** `tick`'s bit 1: the range reached its end. */
export const RANGE_ENDED = 2;

/** The exports of a player, ABI v1: every value is an `i32`. */
interface PlayerExports {
  readonly memory: WebAssembly.Memory;
  abi_version(): number;
  alloc(length: number): number;
  load(pointer: number, length: number): number;
  width(): number;
  height(): number;
  frame_ptr(): number;
  tick(elapsedMs: number): number;
  tag_count(): number;
  tag_name_ptr(index: number): number;
  tag_name_len(index: number): number;
  set_tag(index: number): number;
  set_loop(mode: number): number;
  seek(frame: number): number;
  frame_index(): number;
  title_ptr(): number;
  title_len(): number;
}

/** One player instance playing one payload, through the calls of ABI v1. */
export class PlayerInstance {
  readonly width: number;
  readonly height: number;
  readonly title: string;
  /** The tag names, by index. */
  readonly tags: readonly string[];
  /** The framebuffer, as the canvas takes it: it never moves once loaded. */
  readonly frame: ImageData;
  readonly #player: PlayerExports;
  /** The animation frame the current range starts on. */
  #rangeStart: number;

  private constructor(player: PlayerExports) {
    const buffer = player.memory.buffer;
    this.#player = player;
    this.width = player.width();
    this.height = player.height();
    const pixels = new Uint8ClampedArray(buffer, player.frame_ptr(), this.width * this.height * 4);
    this.frame = new ImageData(pixels, this.width, this.height);
    this.title = decode(buffer, player.title_ptr(), player.title_len());
    this.tags = Array.from({ length: player.tag_count() }, (_, index) =>
      decode(buffer, player.tag_name_ptr(index), player.tag_name_len(index)),
    );
    this.#rangeStart = player.frame_index();
  }

  /** Instantiates the player of an export and loads its payload; throws on any refusal. */
  static async create({ module, payload }: CompiledExport): Promise<PlayerInstance> {
    const { exports } = await WebAssembly.instantiate(module, {});
    const player = exports as unknown as PlayerExports;
    const version = player.abi_version();
    if (version !== ABI_VERSION) throw Error('ABI version ' + version);
    load(player, new Uint8Array(payload));
    return new PlayerInstance(player);
  }

  /** Plays the tag at `index`, or the default range — the first tag, else the whole animation. */
  showTag(index: number): void {
    const player = this.#player;
    player.set_tag(index < 0 && !this.tags.length ? WHOLE_ANIMATION : Math.max(index, 0));
    this.#rangeStart = player.frame_index();
  }

  /** Sets the loop mode: `0` the range's own, `1` loop, `2` once. */
  setLoop(mode: number): void {
    this.#player.set_loop(mode);
  }

  /** Shows a frame of the current range and restarts it; `false` when the range has none. */
  seek(frame: number): boolean {
    return !this.#player.seek(frame);
  }

  /** Advances playback; returns `tick`'s flags. */
  tick(elapsedMs: number): number {
    return this.#player.tick(elapsedMs);
  }

  /** Whether a range that reached its end stayed there, played once, rather than looping. */
  hasStopped(): boolean {
    return this.#player.frame_index() !== this.#rangeStart;
  }
}

function load(player: PlayerExports, payload: Uint8Array): void {
  const pointer = player.alloc(payload.length);
  if (!pointer) refuse(OUT_OF_MEMORY);
  // `alloc` may grow the memory: its buffer is read after the call.
  new Uint8Array(player.memory.buffer, pointer, payload.length).set(payload);
  const status = player.load(pointer, payload.length);
  if (status) refuse(status);
}

/** Throws the ABI v1 status that refused the payload, listed in crates/format/README.md. */
function refuse(status: number): never {
  throw Error('status ' + status);
}

function decode(buffer: ArrayBuffer, pointer: number, length: number): string {
  return new TextDecoder().decode(new Uint8Array(buffer, pointer, length));
}
