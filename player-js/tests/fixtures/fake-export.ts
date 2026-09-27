import { compileFakePlayer } from './fake-player';

/** An RGBA colour: the fake player fills the whole frame with it. */
export type Color = readonly [number, number, number, number];

export interface FakeFrame {
  readonly duration: number;
  readonly color: Color;
}

export interface FakeTag {
  readonly name: string;
  readonly first: number;
  readonly last: number;
  readonly isOnce?: boolean;
}

/** An animation in the fake player's payload, described in `fake-player.wat`. */
export interface FakeAnimation {
  readonly width?: number;
  readonly height?: number;
  readonly title?: string;
  readonly frames: readonly FakeFrame[];
  readonly tags?: readonly FakeTag[];
  /** Replaces `LPIX`, so that `load` refuses the payload with status 1. */
  readonly magic?: string;
}

const SECTION_NAME = 'life-pixel';
const CUSTOM_SECTION_ID = 0;
const DEFAULT_SIDE = 4;

/** Builds an export of the fake player: the module followed by the `life-pixel` section. */
export async function fakeExport(animation: FakeAnimation, abiVersion = 1): Promise<Uint8Array> {
  const player = await compileFakePlayer(abiVersion);
  const name = new TextEncoder().encode(SECTION_NAME);
  const content = [...leb128(name.length), ...name, ...fakePayload(animation)];
  return Uint8Array.from([...player, CUSTOM_SECTION_ID, ...leb128(content.length), ...content]);
}

function fakePayload(animation: FakeAnimation): number[] {
  const encoder = new TextEncoder();
  const title = encoder.encode(animation.title ?? '');
  const tags = animation.tags ?? [];
  const bytes = [...encoder.encode(animation.magic ?? 'LPIX')];
  bytes.push(...u16(animation.width ?? DEFAULT_SIDE), ...u16(animation.height ?? DEFAULT_SIDE));
  bytes.push(...u16(animation.frames.length), ...u16(tags.length), ...u16(title.length), ...title);
  for (const frame of animation.frames) bytes.push(...u16(frame.duration), ...frame.color);
  for (const tag of tags) {
    const name = encoder.encode(tag.name);
    bytes.push(...u16(tag.first), ...u16(tag.last), tag.isOnce ? 1 : 0, name.length, ...name);
  }
  return bytes;
}

function u16(value: number): number[] {
  return [value & 0xff, value >> 8];
}

function leb128(value: number): number[] {
  const bytes = [];
  do {
    const low = value & 0x7f;
    value >>>= 7;
    bytes.push(value ? low | 0x80 : low);
  } while (value);
  return bytes;
}
