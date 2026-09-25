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

1. **Validate** the document with `core` — the same rules as the editor and the server.
2. **Flatten** the layers of each frame, keeping palette indices.
3. **Encode** the frames (run-length, then delta against the previous frame) into a `format`
   payload: a header (magic bytes, format version, player ABI version), the palette(s), the
   frames and their durations, the tags.
4. **Append** the payload to the prebuilt player as a WebAssembly custom section named
   `life-pixel`.

Custom sections may follow the last section of a module, so step 4 is a plain concatenation:
the module stays valid and its code stays exactly the code we built and reviewed. A user's
content can only ever be data, read by a bounds-checked decoder. There is no Rust toolchain in
production, an export is deterministic, and the same compiler runs in the browser, on the
server, in the CLI and on the desktop.

The player embedded in the compiler is built from the same commit, reproducibly; CI checks its
hash and its size.

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
