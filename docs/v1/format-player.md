# Export pipeline — F2 (format types), P1, P2, L1, C1, S1

An export is the prebuilt player plus a payload appended as a custom section (D12). This file
fixes the payload, the player's ABI, the player, the loader, the snippets and the compiler step
that joins them. What ships in users' apps — `crates/format`, `crates/player`, `player-js` — is
MIT and depends on no AGPL code (D15).

## Payload v1

All integers are little-endian and unsigned; lengths precede their content; there is no padding.

### Layout

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 4 | magic | `4C 50 49 58` (`LPIX`) |
| 4 | 2 | format version | `1` |
| 6 | 2 | ABI version | `1` |
| 8 | 2 | width | 1 to `MAX_SIDE` |
| 10 | 2 | height | 1 to `MAX_SIDE` |
| 12 | 2 | frame count | 1 to `MAX_FRAMES` |
| 14 | 2 | reserved | `0` |

Then, in this order:

| Section | Encoding | Rule |
|---|---|---|
| palette | `u16` count, then count × 4 bytes R, G, B, A | 1 to 256 entries; entry 0 is `00 00 00 00` |
| title | `u16` length, then UTF-8 bytes | up to `MAX_TITLE_BYTES`; valid UTF-8 |
| tags | `u16` count, then per tag: `u16` first frame, `u16` last frame, `u8` loop mode, `u8` name length, name bytes | up to `MAX_TAGS`; first ≤ last < frame count; loop mode 0 = loop, 1 = once; name 1 to `MAX_TAG_NAME_BYTES` bytes of valid UTF-8 |
| frames | per frame: `u16` duration in ms, `u8` kind, `u32` data length, data | duration ≥ 1; kind 0 = key, 1 = delta; frame 0 is a key frame |

The payload ends with the last frame's data: a trailing byte is an error.

### Frame data

A frame's data is a sequence of operations over its `width × height` palette indices, row by row
from the top-left pixel. Each operation starts with one byte:

| Bits 7–6 | Operation | Followed by | Effect |
|---|---|---|---|
| `00` | SKIP | nothing | the next *count* pixels keep the previous frame's indices — delta frames only |
| `01` | RUN | one index byte | the next *count* pixels take that index |
| `10` | LITERAL | *count* index bytes | the next *count* pixels take those indices |
| `11` | — | — | invalid |

Bits 5–0 hold `n`: *count* is `n + 1` when `n < 63`; when `n = 63`, a LEB128 varint follows the
operation byte (at most 5 bytes) and *count* is `64 +` its value. The counts of a frame add up to
exactly `width × height`; every index is below the palette count.

### Encoding

The encoder is deterministic: the same animation always gives the same bytes.

- Pixels are flattened before encoding (`core` composites the layers), so a payload knows no layer.
- Within a frame, a run of 3 or more equal indices becomes a RUN; anything else accumulates into a
  LITERAL. In a delta frame, 2 or more unchanged pixels become a SKIP, the rest is encoded as in a
  key frame.
- Frame 0, the first frame of every tag, and any frame whose delta encoding is not smaller than its
  key encoding are key frames; the others are delta frames.

### Bounds

The decoder's bounds — `life-pixel-format::bounds` — are wider than the domain limits of `core`
([core.md](core.md#limits)), which a test of `compiler` checks.

| Bound | Value |
|---|---|
| `MAX_SIDE` | 2,048 |
| `MAX_FRAMES` | 4,096 |
| `MAX_TAGS` | 255 |
| `MAX_TITLE_BYTES` | 1,024 |
| `MAX_TAG_NAME_BYTES` | 64 |
| `MAX_PAYLOAD_BYTES` | 64 MiB |

## ABI v1

The player exports these functions and its `memory`, and imports nothing. Every value is a 32-bit
integer. One module instance plays one payload.

| Export | Signature | Effect |
|---|---|---|
| `abi_version` | `() -> u32` | `1` |
| `alloc` | `(len) -> ptr` | reserves `len` bytes for the payload and returns their address, `0` if it cannot; once per instance |
| `load` | `(ptr, len) -> status` | parses and checks the whole payload written at `ptr`, then shows the first frame of the initial range |
| `width`, `height` | `() -> u32` | the canvas size |
| `frame_ptr` | `() -> ptr` | the framebuffer: `width × height × 4` bytes of RGBA, rows top to bottom, alpha not premultiplied — what `ImageData` expects |
| `tick` | `(elapsed_ms) -> flags` | advances playback; bit 0: the framebuffer changed; bit 1: the range reached its end |
| `tag_count` | `() -> u32` | number of tags |
| `tag_name_ptr`, `tag_name_len` | `(index) -> u32` | the tag's UTF-8 name; `0` for an index out of range |
| `set_tag` | `(index) -> status` | plays tag `index`, or the whole animation for `0xFFFFFFFF`; shows its first frame |
| `set_loop` | `(mode) -> status` | `0` the range's own mode, `1` loop, `2` once |
| `seek` | `(frame) -> status` | shows frame `frame` of the current range, counted from its first frame |
| `frame_index` | `() -> u32` | the animation frame shown |
| `title_ptr`, `title_len` | `() -> u32` | the UTF-8 title |

Statuses: `0` done; `1` bad magic; `2` unknown format version; `3` unknown ABI version; `4`
malformed payload; `5` beyond a bound; `6` out of memory; `7` called before a successful `load`,
or `alloc` and `load` called twice; `8` argument out of range.

### Playback

- The **range** is a tag's frames, or the whole animation. After `load`, it is tag 0 when the
  animation has tags, the whole animation otherwise; the whole animation loops.
- `tick` adds `elapsed_ms` to the time spent on the current frame, then advances while that time
  covers the frame's duration. Leaving the range's last frame sets bit 1: a looping range goes back
  to its first frame, a range played once stays on its last frame and stops — later ticks return
  `0`. For a looping range, the elapsed time is first reduced modulo the range's total duration, so
  that a long pause never loops thousands of times.
- `load`, `set_tag` and `seek` draw the framebuffer before returning: the loader redraws after
  calling them. `set_tag` and `seek` restart a stopped range; `set_loop` applies at once.
- The player decodes forward from the nearest key frame at or before the frame to show, and applies
  a single delta when the next frame follows the one shown.

## Rust API

`crates/format` (F2 types, P1 bodies) is `#![no_std]` and allocation-free, except the encoder,
behind the `encode` feature, which uses `alloc`.

```rust
pub const MAGIC: [u8; 4] = *b"LPIX";
pub const FORMAT_VERSION: u16 = 1;
pub const ABI_VERSION: u16 = 1;
pub mod bounds { /* the constants of the Bounds table */ }

pub struct Rgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }
pub enum LoopMode { Loop, Once }
pub enum FrameKind { Key, Delta }
pub struct Tag<'a> { pub name: &'a str, pub first: u16, pub last: u16, pub loop_mode: LoopMode }
pub struct Frame<'a> { pub duration_ms: u16, pub kind: FrameKind, pub data: &'a [u8] }

/// A payload whose every byte has been checked; it borrows the bytes it was parsed from.
pub struct Payload<'a> { /* private */ }
impl<'a> Payload<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Payload<'a>, DecodeError>;
    pub fn width(&self) -> u16;
    pub fn height(&self) -> u16;
    pub fn palette_len(&self) -> u16;
    pub fn palette_entry(&self, index: u8) -> Option<Rgba>;
    pub fn title(&self) -> &'a str;
    pub fn tag_count(&self) -> u16;
    pub fn tag(&self, index: u16) -> Option<Tag<'a>>;
    pub fn frame_count(&self) -> u16;
    pub fn frames(&self) -> impl Iterator<Item = Frame<'a>> + 'a;
}

/// Applies a frame's operations to `indices`, `width × height` palette indices.
pub fn apply_frame(frame: &Frame<'_>, indices: &mut [u8], palette_len: u16)
    -> Result<(), DecodeError>;

pub enum DecodeError { BadMagic, UnknownFormatVersion, UnknownAbiVersion, Malformed, BeyondBound }
impl DecodeError { pub fn status(&self) -> u32 { /* 1 to 5, as in ABI v1 */ } }

#[cfg(feature = "encode")]
pub struct AnimationData {
    pub width: u16, pub height: u16, pub palette: Vec<Rgba>, pub title: String,
    pub tags: Vec<TagData>, pub frames: Vec<FrameData>,
}
#[cfg(feature = "encode")]
pub struct TagData { pub name: String, pub first: u16, pub last: u16, pub loop_mode: LoopMode }
#[cfg(feature = "encode")]
pub struct FrameData { pub duration_ms: u16, pub indices: Vec<u8> }
#[cfg(feature = "encode")]
pub fn encode(animation: &AnimationData) -> Result<Vec<u8>, EncodeError>;
#[cfg(feature = "encode")]
pub enum EncodeError { BeyondBound, Inconsistent } // a frame of the wrong size, a tag out of range
```

`Payload::parse` walks every frame's operations without writing pixels, so a parsed payload never
fails later; `apply_frame` still returns a `Result` rather than trusting it.

## P1 — Format codec

- The parser and `apply_frame` in `src/decode/`, the encoder in `src/encode/`, the LEB128 varint in
  `src/varint.rs`; no `unsafe`, no panic on any input, no allocation when decoding.
- **Tests**: round trip of generated animations — one pixel, full canvas, 256 colours, maximum
  frames, long runs that need the varint, frames identical to their predecessor —, including the
  choice between key and delta frames; one test per error: bad magic, each unknown version, each
  bound exceeded, truncation inside every section, a trailing byte, an index at the palette count,
  counts that fall short of or pass `width × height`, a SKIP in a key frame, operation `11`, a
  varint of 6 bytes, a tag whose last frame is the frame count, invalid UTF-8 in the title and in
  a tag name.
- **Fixture**: `crates/format/tests/fixtures/v1/` holds `sample.lpix`, a payload of 3 frames with
  2 tags, and `sample.expected.json`, its decoded header, tags and the indices of each frame. A test
  decodes it; the fixture is never changed, only new versions added beside it.
- **Fuzzing**: `crates/format/fuzz/` is a cargo-fuzz project outside the workspace, with the target
  `decode`: `Payload::parse`, then `apply_frame` on every frame of what parses. `_verify.yml` runs
  it 60 seconds on each pull request; `.github/workflows/fuzz.yml` runs it 30 minutes every week
  and keeps the corpus as an artifact.

## P2 — Player

`crates/player`: `crate-type = ["cdylib", "rlib"]`, `#![cfg_attr(target_arch = "wasm32", no_std)]`,
MIT, one dependency: `life-pixel-format`. On the host the crate keeps `std`, so that
`cargo clippy --workspace --all-targets` and its tests build there; the ABI, the allocator and the
panic handler exist only on `wasm32`, and every `wasm32` build — `build-player`, the compiler's
build script — proves that the rest needs no `std`. Its own lint table repeats the workspace's, with
`unsafe_code = "deny"` and `clippy::undocumented_unsafe_blocks = "deny"`: the few items that need
`unsafe` carry `#[allow(unsafe_code)]` and a `// SAFETY:` comment.

| File | Content |
|---|---|
| `src/playback.rs` | the range, time and position rules of [Playback](#playback), pure and tested on the host |
| `src/frames.rs` | the frame table, the key frames, the index buffer, decoding with `apply_frame`, palette to RGBA |
| `src/player.rs` | `Player`: the payload, the buffers, the playback; one method per export, returning statuses |
| `src/abi.rs` | `wasm32` only: the exported functions, each a one-line call into the single `Player` |
| `src/allocator.rs` | `wasm32` only: a bump allocator over `core::arch::wasm32::memory_grow`, which never frees |

- The single `Player` lives in a `static` wrapper whose `Sync` implementation is justified by the
  module having one thread. `load` allocates everything the payload needs, so memory never grows
  during playback.
- A `#[panic_handler]` executes `unreachable`: the module traps and imports nothing.
- `xtask/src/player_build.rs` holds one function, shared with the compiler's build script: the
  command that builds `life-pixel-player` for `wasm32-unknown-unknown` with the `player` profile
  into a given target directory, with `--remap-path-prefix` for the workspace and `CARGO_HOME`.
  `cargo xtask build-player` runs it into `target/player/`; with `--check`, it also verifies the
  hash against `crates/player/player.sha256`, the 16 KiB budget, and plays the
  [fixture](#p1--format-codec) in `wasmi`, comparing each frame's RGBA with the expected indices
  through the palette; `--write-hash` updates the hash file, which every change of the player or
  the format updates in the same commit.
- **Tests**: on the host, playback — frame advance, several frames in one tick, loop and once,
  bit 1 on each cycle, the modulo on long pauses, `set_tag`, `seek`, `set_loop`, the whole
  animation without tags —, and every export through `Player` for statuses and call order; in
  `wasmi`, through `--check`.

## L1 — Loader and element

`player-js/` is the npm package `@life-pixel/player`, MIT, without runtime dependency.

| Path | Content |
|---|---|
| `src/index.ts` | defines `<life-pixel>` unless already defined |
| `src/life-pixel-element.ts` | the element |
| `src/player-instance.ts` | instantiation and the ABI v1 calls |
| `src/module-cache.ts` | one compiled module and payload per absolute URL |
| `life-pixel.js` | the built loader: esbuild bundle, ES module, minified, `es2022` — **committed** |
| `life-pixel.d.ts` | its types, written by hand, including the `HTMLElementTagNameMap` entry — committed |
| `snippets/` | the [snippet templates](#snippets) |
| `tests/` | Playwright tests; `tests/fixtures/fake-player.wat` |
| `tools/check-size.mjs` | fails beyond 2,048 bytes gzipped at level 9 |

The build is committed so that the compiler embeds it and the app ships it without running npm;
CI rebuilds it and fails on any difference. `package.json`: `"type": "module"`, `"exports"` with
`types` and `default`, `"files": ["life-pixel.js", "life-pixel.d.ts", "snippets"]`,
`"sideEffects": ["./life-pixel.js"]`, `"publishConfig": { "access": "public" }`, version `1.0.0`,
scripts `build`, `test` and `size`.

### The element

| Attribute | Default | Effect |
|---|---|---|
| `src` | — | the `.wasm` export; changing it loads the new one |
| `tag` | the first tag | the tag played, by name; empty or unknown: the default, with a console warning when unknown |
| `autoplay` | on | `autoplay="false"` waits for `play()` |
| `loop` | the tag's mode | `loop` or `loop="true"`: loop; `loop="false"`: once |
| `alt` | the animation's title | the accessible name; `alt=""` marks the element decorative |
| `motion` | `auto` | `always` plays even when reduced motion is preferred |

- Properties `src` and `tag` reflect their attributes; `playing` is read-only; methods `play()`,
  `pause()` and `seek(frame)`; events `load` (first frame drawn), `error` and `tagend`.
- Loading: `WebAssembly.compileStreaming(fetch(src))`, and on a `TypeError` — a server that does not
  send `application/wasm` — `fetch`, `arrayBuffer`, `WebAssembly.compile`. The payload is
  `WebAssembly.Module.customSections(module, "life-pixel")[0]`. The instance gets empty imports;
  `abi_version()` must be `1`; then `alloc`, a copy of the payload into memory, `load`. Any failure
  draws nothing, logs one `console.warn` naming the status, and fires `error`.
- Drawing: a shadow root holds a `<canvas>` at the animation's native size, styled `width: 100%;
  height: auto; image-rendering: pixelated`; the host is `display: inline-block`. After a change, an
  `ImageData` over `memory.buffer` at `frame_ptr()` is put on the canvas.
- Timing: one `requestAnimationFrame` loop per playing element passes the elapsed milliseconds to
  `tick`, capped at 1,000 ms. The loop stops while an `IntersectionObserver` reports the element
  off-screen, while `document.hidden`, while paused, and after a range played once has ended.
- Reduced motion: when `prefers-reduced-motion: reduce` matches and `motion` is not `always`, the
  element shows the first frame of its range and does not autoplay; `play()` still plays.
- Accessibility: `role="img"` and `aria-label` from `alt`, else the title — set in
  `connectedCallback`, never in the constructor; `alt=""` sets `aria-hidden="true"` instead.

**Tests** (Playwright, files served through `page.route`, no server): first frame drawn and read
back through `getImageData`; tag by attribute and by property; `tagend` after a range played once
and on every loop; `loop` overriding the tag; pause off-screen and in a hidden tab; reduced motion,
and `motion="always"`; accessible name from `alt`, from the title, and `alt=""`; the fallback on a
wrong MIME type; `error` and nothing drawn for an unknown ABI; one fetch for two elements with the
same `src`. They run against `fake-player.wat`, compiled at test time with the `wabt` package; C1
later adds a run against real exports.

## Snippets

The templates live in `player-js/snippets/` — MIT, shipped in the package — and `compiler`
renders them for the export dialog and for MCP: one source for both.

| File | Content |
|---|---|
| `html.html` | `<script type="module" src="{{loader}}"></script>` and `<life-pixel src="{{src}}"{{tagAttribute}} alt="{{alt}}"></life-pixel>` |
| `angular.ts` | `import '@life-pixel/player';` in `main.ts`; a standalone component `{{className}}` with `schemas: [CUSTOM_ELEMENTS_SCHEMA]` and the element in its template |
| `react.tsx` | `import '@life-pixel/player';`, a component `{{className}}` returning the element with `alt={{altExpression}}`, and the JSX declaration a TypeScript project needs |
| `vue.vue` | `<script setup>` importing the package, the element in `<template>`, and a comment with the `isCustomElement` option for Vite |

Placeholders: `{{loader}}`, `{{src}}`, `{{tagAttribute}}` (` tag="…"`, or nothing without tags),
`{{alt}}`, `{{altExpression}}` (a JavaScript string literal), `{{className}}` (the file stem in
PascalCase, followed by `Animation`). Values are escaped for the snippet's language: HTML attribute
escaping, JavaScript string escaping for `altExpression`.

## C1 — WASM export

`crates/compiler` (created by C2) gains:

| Path | Content |
|---|---|
| `build.rs` | builds the player with `xtask/src/player_build.rs`, included through `include!`, into `$OUT_DIR/player-target`, after removing `RUSTC_WORKSPACE_WRAPPER`, `RUSTC_WRAPPER`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTFLAGS`, `CARGO_TARGET_DIR` and `CARGO_BUILD_TARGET` from the command's environment; `rerun-if-changed` on `crates/player`, `crates/format`, `xtask/src/player_build.rs`, the root `Cargo.toml` and `Cargo.lock` |
| `src/player.rs` | `PLAYER_WASM: &[u8]`, the built player; `LOADER_JS: &str`, `include_str!` of `player-js/life-pixel.js` |
| `src/wasm_export.rs` | `pub fn export_wasm(animation: &Animation) -> Result<Vec<u8>, ExportError>` |
| `src/snippets.rs` | `pub fn render_snippet(framework: Framework, input: &SnippetInput) -> String`, templates through `include_str!`; `Framework` is `Html`, `Angular`, `React` or `Vue` (`html`, `angular`, `react`, `vue`); `SnippetInput` holds `src`, `loader`, `tag: Option<String>`, `alt` and `file_stem` |

`export_wasm` composites every frame with `core`, builds `format::AnimationData` — tags with their
first and last frames and loop modes, the palette, the title —, encodes it, and appends to
`PLAYER_WASM` the custom section: byte `0x00`, the LEB128 size of what follows, the LEB128 length
of the name, `life-pixel`, the payload.

**Tests**: golden files in `crates/compiler/tests/golden/` — a few small animations as documents
with grid cels, and their exports —, compared byte for byte; each export played in `wasmi` shows,
frame after frame, the RGBA of `core::render::rgba`; tag names and loop modes come back through the
ABI; the domain limits of `core` fit within the format's bounds; the same document exported twice
gives the same bytes. C1 also adds `player-js/tests/golden-exports.spec.ts`, which plays those
exports through the loader in a browser.

## S1 — Size checkpoint

- `samples/` (scope `samples`) holds four animations drawn for the project, as documents whose
  cels are text grids ([core.md](core.md#text-grid)), and a `README.md` dedicating them to the
  public domain under CC0-1.0:

| Sample | Size | Frames | Tags | Colours |
|---|---|---|---|---|
| `mascot-wave` | 32 × 32 | 8 | `idle`, `wave` | 12 |
| `loader-dots` | 16 × 16 | 6 | — | 4 |
| `hero-run` | 48 × 48 | 16 | `idle`, `run`, `jump` (once) | 24 |
| `empty-state` | 128 × 96 | 12 | — | 16 |

- `cargo xtask measure-sizes` exports each sample as WASM, GIF and APNG, and as lossless animated
  WebP through `img2webp` from its PNG frames, for the measurement only; it measures each raw,
  gzipped at level 9 and with brotli at level 11, plus the player alone and the loader; it writes
  the table between `<!-- sizes:start -->` and `<!-- sizes:end -->` in [export.md](../export.md),
  and a demo in `target/demo/` — the loader, the exports, and a page from `samples/demo.html` that
  plays every sample and switches their tags.
- Once measured, the budgets of export.md are confirmed or revised, with the reason, and the sizes
  quoted in [pricing.md](../pricing.md) checked against the measure.
