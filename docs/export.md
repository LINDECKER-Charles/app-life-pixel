# Export and integration

An export has to be three things at once: light, dependency-free, and safe to run inside
someone else's app. This document describes how the design gets there. The compilation
mechanism is settled (D12 in [decisions.md](decisions.md)); the budgets are drafts, settled by
the M1 prototype. The payload and the player ABI are specified byte by byte in
[crates/format/README.md](../crates/format/README.md), their reference.

Everything an export puts into an app — the player, the loader, the integration snippets — is
MIT-licensed (D15): the snippets are templates kept with the loader in `player-js`.

## What an export produces

| Artefact | Use |
|---|---|
| `<name>.wasm` | the animation: the prebuilt player plus the animation's payload, self-contained |
| `life-pixel.js` | the loader that defines `<life-pixel>`; one copy serves every animation of a site. Also on npm as `@life-pixel/player` |
| `<name>.js` (single-file option) | loader and animation inlined in one ES module, for "drop one file" integrations |
| GIF, APNG, sprite sheet + JSON, PNG frames | the classic formats, when no control from code is needed or a tool expects them |

Exports are compiled on demand, in milliseconds, and never stored: they do not count against the
storage quota.

## Compiling: data, not code

1. **Validate** the document with `core` — the same rules as the editor and the server. Reading a
   document validates it, so the compiler's `export_wasm` takes an `Animation`, which is valid
   by construction.
2. **Flatten** the layers of each frame with `core`'s compositing — the visible layers from
   bottom to top, index 0 transparent —, keeping palette indices. Palette entry 0, which `core`
   only requires fully transparent, is written `00 00 00 00`, as the payload requires.
3. **Encode** the frames (run-length, then delta against the previous frame) into a `format`
   payload: a header (magic bytes, format version, player ABI version, size, frame count), the
   palette, the title, the tags with their first and last frames and loop modes, then the frames
   and their durations. The domain limits of `core` fit within the format's bounds, so every
   animation the editor accepts encodes.
4. **Append** the payload to the prebuilt player as a WebAssembly custom section named
   `life-pixel`: byte `0x00`, the LEB128 size of what follows, the LEB128 length of the name,
   `life-pixel`, the payload.

Custom sections may follow the last section of a module, so step 4 is a plain concatenation:
the module stays valid and its code stays exactly the code we built and reviewed. A user's
content can only ever be data, read by a bounds-checked decoder. There is no Rust toolchain in
production, an export is deterministic, and the same compiler runs in the browser, on the
server, in the CLI and on the desktop.

The player embedded in the compiler is built from the same commit, reproducibly: the compiler's
build script runs the function `cargo xtask build-player` runs (`xtask/src/player_build.rs`),
into its own target directory, with the workspace and `CARGO_HOME` paths remapped and without
the outer build's compiler wrappers, flags, target or target directory. Whatever builds the
compiler — `cargo clippy`, a build for `wasm32-unknown-unknown` —, the module hashes to
`crates/player/player.sha256`, which a test of the compiler checks; CI checks the hash and the
size of the player. The loader, `player-js/life-pixel.js`, is embedded as committed. The golden
exports of `crates/compiler/tests/golden/` are played frame by frame in `wasmi` against
`core`'s rendering, and in a browser through the loader.

## Playing

1. The loader compiles the module with `WebAssembly.compileStreaming`, falling back to
   `arrayBuffer` when the server does not send `application/wasm`.
2. It reads the `life-pixel` section (`WebAssembly.Module.customSections`), instantiates the
   module, copies the payload into its memory and calls `load`.
3. On each animation frame it calls `tick` with the elapsed time. When the image changed, it
   copies the framebuffer into a canvas at the animation's native size.
4. CSS scales the canvas with `image-rendering: pixelated`: pixel-exact at any size, at no
   resampling cost.

Player ABI, version 1: the player exports these functions and its `memory`, and imports nothing;
every value is a 32-bit integer. The statuses and the playback rules are in
[crates/format/README.md](../crates/format/README.md#player-abi-v1).

| Export | Signature | Effect |
|---|---|---|
| `abi_version` | `() -> u32` | `1` |
| `alloc` | `(len) -> ptr` | reserves `len` bytes for the payload and returns their address, `0` if it cannot; once per instance |
| `load` | `(ptr, len) -> status` | parses and checks the whole payload written at `ptr`, then shows the first frame of the initial range; a non-zero status refuses it |
| `width`, `height` | `() -> u32` | the canvas size |
| `frame_ptr` | `() -> ptr` | the framebuffer: `width × height × 4` bytes of RGBA, rows top to bottom, alpha not premultiplied |
| `tick` | `(elapsed_ms) -> flags` | advances playback; bit 0: the framebuffer changed; bit 1: the range reached its end |
| `tag_count` | `() -> u32` | number of tags |
| `tag_name_ptr`, `tag_name_len` | `(index) -> u32` | the tag's UTF-8 name; `0` for an index out of range |
| `set_tag` | `(index) -> status` | plays tag `index`, or the whole animation for `0xFFFFFFFF`; shows its first frame |
| `set_loop` | `(mode) -> status` | `0` the range's own mode, `1` loop, `2` once |
| `seek` | `(frame) -> status` | shows frame `frame` of the current range, counted from its first frame |
| `frame_index` | `() -> u32` | the animation frame shown |
| `title_ptr`, `title_len` | `() -> u32` | the UTF-8 title |

The element also:

- pauses when it leaves the viewport and when the tab is hidden;
- honours `prefers-reduced-motion`: it shows the poster frame and does not autoplay, unless the
  page sets `motion="always"`;
- exposes `role="img"` and an accessible name taken from `alt`, or from the animation's title.

## Integration

```html
<script type="module" src="/assets/life-pixel.js"></script>
<life-pixel src="/assets/mascot.wasm" tag="idle" alt="The mascot waving"></life-pixel>
```

```js
const mascot = document.querySelector('life-pixel');
mascot.tag = 'jump';
mascot.pause();
mascot.seek(0);
mascot.addEventListener('tagend', () => { mascot.tag = 'idle'; });
```

| Attribute | Default | Effect |
|---|---|---|
| `src` | — | the `.wasm` export |
| `tag` | first tag | the named range to play |
| `autoplay` | present | start when visible |
| `loop` | from the tag | override the tag's loop mode |
| `alt` | the animation's title | accessible name |
| `motion` | `auto` | `always` ignores `prefers-reduced-motion` |

- **Frameworks**: a custom element works in Angular (`CUSTOM_ELEMENTS_SCHEMA`), React 19, Vue
  (`isCustomElement`) and Svelte. The export dialog and the MCP tool `get_embed_snippet` produce
  the snippet for each.
- **Content Security Policy**: a page with a strict CSP needs `'wasm-unsafe-eval'` in
  `script-src` to compile WebAssembly — without allowing JavaScript `eval`.
- **Webviews**: Capacitor, Tauri, Electron, React Native WebView and Flutter webviews play
  exports as a browser does. A native app without a webview would need a native player, which is
  not planned (see [product.md](product.md)).

## Budgets

| Item | Initial budget |
|---|---|
| player `.wasm`, without payload | ≤ 16 KiB |
| `life-pixel.js`, minified and gzipped | ≤ 2 KiB |

CI fails when a budget is exceeded. The M1 prototype confirms or revises these numbers once;
after that, raising a budget is a design discussion, never a silent edit.

**Confirmed by S1's size checkpoint** ([Measured sizes](#measured-sizes)): the player measures
11,838 B (≤ 16 KiB, 72% of budget) and the loader 1,926 B gzipped (≤ 2 KiB, 94% of budget).
Neither budget is exceeded, so both are kept as initially set; the loader's headroom is the
tighter of the two and is worth watching as `player-js/` grows.

## Measured sizes

`cargo xtask measure-sizes` (S1, [docs/v1/format-player.md](v1/format-player.md#s1--size-checkpoint))
compiles the four animations of [`samples/`](../samples/README.md) as WASM, GIF and APNG through
the compiler, and as a lossless animated WebP through `img2webp` from their PNG frames — WebP
measurement only, the format is not exported by the product (see
[Classic formats](#classic-formats)) — then measures each raw, gzipped at level 9 and with
brotli at level 11, plus the player alone and the loader. Run twice, the table is identical: the
compiler, `img2webp` and the two compressors are deterministic for a fixed input.

<!-- sizes:start -->
| Artefact | Raw | Gzip 9 | Brotli 11 |
|---|---|---|---|
| `mascot-wave.wasm` | 13501 B | 6231 B | 5736 B |
| `mascot-wave.gif` | 1653 B | 852 B | 793 B |
| `mascot-wave.apng` | 2313 B | 1255 B | 1175 B |
| `mascot-wave.webp` | 1552 B | 1281 B | 1189 B |
| `loader-dots.wasm` | 12139 B | 5623 B | 5138 B |
| `loader-dots.gif` | 321 B | 241 B | 205 B |
| `loader-dots.apng` | 738 B | 587 B | 525 B |
| `loader-dots.webp` | 480 B | 321 B | 295 B |
| `hero-run.wasm` | 17448 B | 7926 B | 7200 B |
| `hero-run.gif` | 5066 B | 2644 B | 2580 B |
| `hero-run.apng` | 5992 B | 3693 B | 3556 B |
| `hero-run.webp` | 4576 B | 2270 B | 2184 B |
| `empty-state.wasm` | 13648 B | 6010 B | 5461 B |
| `empty-state.gif` | 5280 B | 2362 B | 2056 B |
| `empty-state.apng` | 4646 B | 4114 B | 4028 B |
| `empty-state.webp` | 1744 B | 1322 B | 1241 B |
| `player.wasm (no payload)` | 11838 B | 5429 B | 4933 B |
| `life-pixel.js (loader)` | 4376 B | 1926 B | 1684 B |
<!-- sizes:end -->

## Versioning

- The payload header carries the format version and the player ABI version. A player refuses an
  unknown version with an error status; the loader then shows nothing and logs why.
- A loader plays every player ABI of its major version. A new ABI major is a new major of
  `@life-pixel/player`.
- A fixture of every released format version is kept in the repository and played in CI, so an
  export made today keeps playing after any update.

## Classic formats

| Format | Notes |
|---|---|
| GIF | exact, since palettes are indexed; partial transparency is lost |
| APNG | full alpha, larger than GIF for long animations |
| Sprite sheet + JSON | frame rectangles, durations and tags; the JSON aims at the layout most game engines already import |
| PNG frames | one image per frame, zipped |
| Animated WebP | later |
