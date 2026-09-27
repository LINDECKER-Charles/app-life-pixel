# life-pixel-format

The versioned binary format of a compiled Life Pixel animation. This document is the reference
for **payload v1** — the bytes an export carries — and **player ABI v1** — the functions the
player exposes to the loader that plays those bytes. Exports live for years inside other
people's apps: what is written here never changes for version 1. A change takes a new format
version, and a player refuses, never misreads, a payload it does not know.

An export is the prebuilt player, `crates/player`, followed by a WebAssembly custom section named
`life-pixel` whose content is the payload ([export.md](../../docs/export.md)).

The crate is `#![no_std]`. Its decoder never allocates and never panics, whatever the input; the
encoder, behind the `encode` feature, uses `alloc`. Like the player and `player-js`, it is under
the [MIT licence](../../LICENSE-MIT) and depends on no other crate.

## Payload v1

### Conventions

- Every integer is unsigned and little-endian: `u8`, `u16` (2 bytes), `u32` (4 bytes).
- A length or a count precedes what it counts. There is no padding and no alignment.
- Strings are UTF-8, without a terminator.
- A payload is a header, then four sections in this order: palette, title, tags, frames. It ends
  with the last byte of the last frame's data.

### Header

16 bytes.

| Offset | Size | Field | Value |
|---|---|---|---|
| 0 | 4 | magic | `4C 50 49 58`, `LPIX` in ASCII |
| 4 | 2 | format version | `1` |
| 6 | 2 | ABI version | `1`: the player ABI the payload is written for |
| 8 | 2 | width, in pixels | 1 to `MAX_SIDE` |
| 10 | 2 | height, in pixels | 1 to `MAX_SIDE` |
| 12 | 2 | frame count | 1 to `MAX_FRAMES` |
| 14 | 2 | reserved | `0` |

### Palette

| Size | Field | Value |
|---|---|---|
| 2 | entry count | 1 to 256 |
| 4 × count | entries | per entry: `u8` red, `u8` green, `u8` blue, `u8` alpha, alpha not premultiplied |

Entry 0 is `00 00 00 00`, fully transparent. A pixel is an index into this palette.

### Title

| Size | Field | Value |
|---|---|---|
| 2 | length, in bytes | 0 to `MAX_TITLE_BYTES` |
| length | title | valid UTF-8 |

### Tags

A tag is a named range of frames.

| Size | Field | Value |
|---|---|---|
| 2 | tag count | 0 to `MAX_TAGS` |

Then, per tag:

| Size | Field | Value |
|---|---|---|
| 2 | first frame | `first ≤ last` |
| 2 | last frame, included | `last < frame count` |
| 1 | loop mode | `0` loop, `1` once |
| 1 | name length, in bytes | 1 to `MAX_TAG_NAME_BYTES` |
| name length | name | valid UTF-8 |

### Frames

Exactly *frame count* frames, one after the other:

| Size | Field | Value |
|---|---|---|
| 2 | duration, in milliseconds | 1 or more |
| 1 | kind | `0` key, `1` delta; frame 0 is a key frame |
| 4 | data length, in bytes | the size of the data that follows |
| data length | data | the frame's operations, below |

### Frame data

A frame's data is a sequence of operations over the frame's `width × height` palette indices,
row by row from the top-left pixel. Each operation starts with one byte:

| Bits 7–6 | Operation | Followed by | Effect |
|---|---|---|---|
| `00` | SKIP | nothing | the next *count* pixels keep the previous frame's indices — delta frames only |
| `01` | RUN | one index byte | the next *count* pixels take that index |
| `10` | LITERAL | *count* index bytes | the next *count* pixels take those indices, in order |
| `11` | — | — | invalid |

Bits 5–0 hold `n`:

- `n` from 0 to 62: *count* is `n + 1`, from 1 to 63;
- `n = 63`: an unsigned LEB128 varint follows the operation byte — 7 bits per byte, least
  significant group first, bit 7 set on every byte but the last, at most 5 bytes — and *count*
  is `64 +` its value. For a RUN, the index byte comes after the varint; for a LITERAL, the
  indices do.

Every pixel is covered exactly once: the counts of a frame add up to exactly `width × height`,
and the operations end exactly at the end of the frame's data. Every index is below the palette's
entry count. A key frame sets every pixel. A delta frame applies to the indices of the frame
before it, frame `i − 1`: SKIP keeps them, RUN and LITERAL replace them.

Examples of operation bytes: `43 01` is a RUN of 4 pixels of index 1; `02` a SKIP of 3 pixels;
`81 05 06` a LITERAL of indices 5 then 6; `7F EC 01 05` a RUN of 300 pixels of index 5 — `EC 01`
is 236, and 64 + 236 = 300.

### Bounds

The decoder's bounds are the constants of `life_pixel_format::bounds`. They are wider than the
domain limits of `life-pixel-core`, so that every animation the editor accepts fits.

| Bound | Value | Limits |
|---|---|---|
| `MAX_SIDE` | 2,048 | width, height |
| `MAX_FRAMES` | 4,096 | frame count |
| `MAX_TAGS` | 255 | tag count |
| `MAX_TITLE_BYTES` | 1,024 | title length |
| `MAX_TAG_NAME_BYTES` | 64 | a tag's name length |
| `MAX_PAYLOAD_BYTES` | 67,108,864 (64 MiB) | the payload's total length |

### Errors

The decoder checks the whole payload before anything plays it — every section, every frame's
operations — and refuses it with the first error it meets:

1. a payload longer than `MAX_PAYLOAD_BYTES` is `BeyondBound`, before any byte is read;
2. then the fields are checked in the order of the bytes. A magic other than `LPIX` is
   `BadMagic`; a format version other than 1 is `UnknownFormatVersion`; an ABI version other than
   1 is `UnknownAbiVersion`;
3. a value above a bound of the table above is `BeyondBound`;
4. every other broken rule is `Malformed`: a field or a section cut short by the end of the
   payload, a byte after the last frame, a zero width, height, frame count, duration or tag name
   length, a non-zero reserved field, a palette of 0 or more than 256 entries, an entry 0 that is
   not `00 00 00 00`, invalid UTF-8, a tag with `first > last` or `last ≥ frame count`, a loop
   mode or a frame kind above 1, a delta frame 0, operation `11`, a SKIP in a key frame, a varint
   longer than 5 bytes, an index at or above the palette's entry count, counts that fall short of
   or pass `width × height`, and operations that do not end exactly at the end of the frame's
   data.

| Error | Player status |
|---|---|
| `BadMagic` | 1 |
| `UnknownFormatVersion` | 2 |
| `UnknownAbiVersion` | 3 |
| `Malformed` | 4 |
| `BeyondBound` | 5 |

### Encoding

The encoder is deterministic: the same animation always gives the same bytes. A decoder does not
rely on the rules below — any payload that follows the rules above plays.

- Pixels are flattened before encoding — `life-pixel-core` composites the layers —, so a payload
  knows no layer.
- Within a frame, a run of 3 or more equal indices becomes a RUN; anything else accumulates into a
  LITERAL. In a delta frame, 2 or more unchanged pixels become a SKIP; the rest is encoded as in a
  key frame.
- Frame 0, the first frame of every tag, and any frame whose delta encoding is not smaller than
  its key encoding are key frames; the others are delta frames.

### Example

A 2 × 2 animation titled `Hi`, with a transparent entry and an opaque red one, two frames of
100 ms and one looping tag `idle` over both. Frame 0 is all red; frame 1 turns its last pixel
transparent. 61 bytes:

```text
0000  4C 50 49 58 01 00 01 00 02 00 02 00 02 00 00 00
0010  02 00 00 00 00 00 FF 00 00 FF 02 00 48 69 01 00
0020  00 00 01 00 00 04 69 64 6C 65 64 00 00 02 00 00
0030  00 43 01 64 00 01 03 00 00 00 02 80 00
```

| Offset | Bytes | Meaning |
|---|---|---|
| 0 | `4C 50 49 58` | magic |
| 4 | `01 00` `01 00` | format version 1, ABI version 1 |
| 8 | `02 00` `02 00` `02 00` `00 00` | width 2, height 2, 2 frames, reserved |
| 16 | `02 00` | 2 palette entries |
| 18 | `00 00 00 00` `FF 00 00 FF` | transparent, opaque red |
| 26 | `02 00` `48 69` | title of 2 bytes: `Hi` |
| 30 | `01 00` | 1 tag |
| 32 | `00 00` `01 00` `00` `04` `69 64 6C 65` | frames 0 to 1, loop, name of 4 bytes: `idle` |
| 42 | `64 00` `00` `02 00 00 00` `43 01` | frame 0: 100 ms, key, 2 bytes: RUN 4 × index 1 |
| 51 | `64 00` `01` `03 00 00 00` `02 80 00` | frame 1: 100 ms, delta, 3 bytes: SKIP 3, LITERAL 1 × index 0 |

Frame 1 is a delta frame: its delta encoding, 3 bytes, is smaller than its key encoding,
`42 01 80 00`.

## Player ABI v1

The player is a WebAssembly module. It exports the functions below and its linear memory, named
`memory`, and imports nothing. Every parameter and result is an `i32`, read as an unsigned 32-bit
integer; a pointer is an offset into `memory`. One module instance plays one payload.

| Export | Signature | Effect |
|---|---|---|
| `abi_version` | `() -> u32` | `1` |
| `alloc` | `(len) -> ptr` | reserves `len` bytes for the payload and returns their address, `0` if it cannot; once per instance |
| `load` | `(ptr, len) -> status` | parses and checks the whole payload written at `ptr`, then shows the first frame of the initial range |
| `width`, `height` | `() -> u32` | the canvas size, in pixels |
| `frame_ptr` | `() -> ptr` | the framebuffer: `width × height × 4` bytes of RGBA, rows top to bottom, alpha not premultiplied — what `ImageData` expects |
| `tick` | `(elapsed_ms) -> flags` | advances playback; bit 0: the framebuffer changed; bit 1: the range reached its end |
| `tag_count` | `() -> u32` | the number of tags |
| `tag_name_ptr`, `tag_name_len` | `(index) -> u32` | the tag's UTF-8 name; `0` for an index out of range |
| `set_tag` | `(index) -> status` | plays tag `index`, or the whole animation for `0xFFFFFFFF`; shows its first frame |
| `set_loop` | `(mode) -> status` | `0` the range's own mode, `1` loop, `2` once |
| `seek` | `(frame) -> status` | shows frame `frame` of the current range, counted from its first frame |
| `frame_index` | `() -> u32` | the animation frame shown, counted from frame 0 of the animation |
| `title_ptr`, `title_len` | `() -> u32` | the UTF-8 title |

### Statuses

| Status | Meaning |
|---|---|
| 0 | done |
| 1 | bad magic |
| 2 | unknown format version |
| 3 | unknown ABI version |
| 4 | malformed payload |
| 5 | beyond a bound |
| 6 | out of memory |
| 7 | called before a successful `load`, or `alloc` and `load` called twice |
| 8 | argument out of range |

Statuses 1 to 5 are the decoder's [errors](#errors).

### Loading

1. The loader compiles the module and reads its `life-pixel` custom section: the payload.
2. It instantiates the module with no import, calls `alloc(len)` with the payload's length, and
   copies the payload into `memory` at the returned address.
3. It calls `load(ptr, len)`: any status but `0` refuses the payload, and the loader shows
   nothing. `alloc` and `load` may grow `memory`, so the loader reads `memory.buffer` again after
   them.
4. It reads `width`, `height` and `frame_ptr` once — the framebuffer never moves —, draws the
   first frame, then calls `tick` with the time elapsed since the previous call, and redraws
   whenever bit 0 is set.

### Playback

- The **range** is a tag's frames, or the whole animation. After `load`, it is tag 0 when the
  animation has tags, the whole animation otherwise; the whole animation loops.
- `tick` adds `elapsed_ms` to the time spent on the current frame, then advances while that time
  covers the frame's duration. Leaving the range's last frame sets bit 1: a looping range goes back
  to its first frame, a range played once stays on its last frame and stops — later ticks return
  `0`. For a looping range, the elapsed time is first reduced modulo the range's total duration,
  so that a long pause never loops thousands of times.
- `load`, `set_tag` and `seek` draw the framebuffer before returning: the loader redraws after
  calling them. `set_tag` and `seek` restart a stopped range; `set_loop` applies at once.
- The player decodes forward from the nearest key frame at or before the frame to show, and
  applies a single delta when the next frame follows the one shown.
- `load` reserves everything the payload needs, so `memory` never grows during playback.

## Rust API

| Item | Role |
|---|---|
| `MAGIC`, `FORMAT_VERSION`, `ABI_VERSION`, `bounds` | the constants above |
| `Payload::parse` | checks a whole payload and borrows it: header, palette, title, tags and frames |
| `apply_frame` | applies a frame's operations to `width × height` indices |
| `DecodeError` and `DecodeError::status` | why a payload is refused, and its player status |
| `encode`, with `AnimationData`, `TagData`, `FrameData` and `EncodeError` | the encoder, behind the `encode` feature |

`cargo doc -p life-pixel-format --all-features --open` documents each item.

## Fixtures and fuzzing

- `tests/fixtures/v1/` holds `sample.lpix`, a payload v1 of 3 frames and 2 tags, and
  `sample.expected.json`: its header, palette, title and tags, and each frame's duration, kind
  and indices once applied. `tests/fixture_v1.rs` decodes the one and compares it with the other.
  A fixture never changes: a new format version adds its own beside it, and the old one keeps
  playing.
- `fuzz/` is a [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) project with a workspace of
  its own. Its target `decode` feeds any bytes to `Payload::parse`, then `apply_frame` to every
  frame of what parses; it needs the nightly toolchain. CI runs it 60 seconds on each pull request
  and 30 minutes every week.

```shell
cargo test -p life-pixel-format --all-features
cd crates/format && cargo +nightly fuzz run decode -- -max_total_time=60
```
