# Web editor — W0, U1 to U6, W1

The editor is `frontend/projects/app`. Every pixel rule runs in `core` through `editor-wasm`, inside
a Web Worker; Angular displays and delegates. Until W1 lands, the interface tasks work against
W0's mock.

```text
components ──► EngineStore (signals) ──► EditorEngine ──► WasmEditorEngine ──► worker ──► editor-wasm
                                              └──────────► MockEditorEngine (tests, before W1)
```

## W0 — Engine interface

`frontend/projects/app/src/app/engine/`:

| File | Content |
|---|---|
| `engine-types.ts` | the types below |
| `editor-engine.ts` | the `EditorEngine` interface and the `EDITOR_ENGINE` injection token |
| `engine-store.ts` | `EngineStore`: the engine's state as a signal, and its commands |
| `testing/mock-editor-engine.ts` | the mock |
| `testing/engine-contract.ts` | `runEngineContract(create: () => Promise<EditorEngine>)`, the shared suite |

```ts
export type LayerId = number;
export type FrameId = number;
export type Color = string; // "#rrggbbaa"
export type LoopMode = 'loop' | 'once';
export interface Point { readonly x: number; readonly y: number }
export interface Area { readonly x: number; readonly y: number; readonly width: number; readonly height: number }
/** The JSON shape of core's Tag: `loop_mode` is serialized as `loop`. */
export interface TagSpec { readonly name: string; readonly first: number; readonly last: number; readonly loop: LoopMode }

/** Mirrors core::edit::Operation field for field (core.md, K2). */
export type EditOperation =
  | { readonly kind: 'paintStroke'; readonly layer: LayerId; readonly frame: FrameId; readonly points: readonly Point[]; readonly index: number }
  | { readonly kind: 'fill'; readonly layer: LayerId; readonly frame: FrameId; readonly at: Point; readonly index: number }
  | { readonly kind: 'line'; readonly layer: LayerId; readonly frame: FrameId; readonly from: Point; readonly to: Point; readonly index: number }
  | { readonly kind: 'rectangle'; readonly layer: LayerId; readonly frame: FrameId; readonly from: Point; readonly to: Point; readonly index: number; readonly filled: boolean }
  | { readonly kind: 'moveSelection'; readonly layer: LayerId; readonly frame: FrameId; readonly area: Area; readonly offset: Point }
  | { readonly kind: 'setPaletteEntry'; readonly index: number; readonly color: Color }
  | { readonly kind: 'addPaletteEntry'; readonly color: Color }
  | { readonly kind: 'removePaletteEntry'; readonly index: number }
  | { readonly kind: 'movePaletteEntry'; readonly from: number; readonly to: number }
  | { readonly kind: 'addLayer'; readonly position: number; readonly name: string }
  | { readonly kind: 'deleteLayer'; readonly layer: LayerId }
  | { readonly kind: 'moveLayer'; readonly layer: LayerId; readonly position: number }
  | { readonly kind: 'renameLayer'; readonly layer: LayerId; readonly name: string }
  | { readonly kind: 'setLayerVisibility'; readonly layer: LayerId; readonly visible: boolean }
  | { readonly kind: 'addFrame'; readonly position: number; readonly durationMs: number }
  | { readonly kind: 'duplicateFrame'; readonly frame: FrameId }
  | { readonly kind: 'deleteFrame'; readonly frame: FrameId }
  | { readonly kind: 'moveFrame'; readonly frame: FrameId; readonly position: number }
  | { readonly kind: 'setFrameDuration'; readonly frame: FrameId; readonly durationMs: number }
  | { readonly kind: 'addTag'; readonly tag: TagSpec }
  | { readonly kind: 'updateTag'; readonly name: string; readonly tag: TagSpec }
  | { readonly kind: 'deleteTag'; readonly name: string }
  | { readonly kind: 'replaceTags'; readonly tags: readonly TagSpec[] }
  | { readonly kind: 'setTitle'; readonly title: string }
  | { readonly kind: 'importImage'; readonly layer: LayerId; readonly frame: FrameId; readonly png: Uint8Array; readonly at: Point }
  | { readonly kind: 'importSpriteSheet'; readonly layer: LayerId; readonly position: number; readonly png: Uint8Array; readonly cellWidth: number; readonly cellHeight: number; readonly durationMs: number };

export interface NewAnimationOptions {
  readonly title: string; readonly width: number; readonly height: number;
  readonly layerName: string; readonly frameDurationMs?: number; readonly palette?: readonly Color[];
}
export interface DocumentSummary {
  readonly title: string; readonly width: number; readonly height: number;
  readonly palette: readonly Color[];
  readonly layers: readonly { readonly id: LayerId; readonly name: string; readonly visible: boolean }[];
  readonly frames: readonly { readonly id: FrameId; readonly durationMs: number }[];
  readonly tags: readonly TagSpec[];
}
export interface EngineState {
  readonly status: 'starting' | 'empty' | 'ready' | 'failed';
  readonly document: DocumentSummary | null;
  readonly canUndo: boolean; readonly canRedo: boolean; readonly hasUnsavedWork: boolean;
  readonly limits: Limits | null; // core's Limits::current()
}
export interface RenderRequest { readonly frame: FrameId; readonly preview?: EditOperation }
export interface RenderedFrame { readonly width: number; readonly height: number; readonly pixels: Uint8ClampedArray }
/** One readonly field per constant of core.md's limits, in camelCase: CANVAS_MAX_SIDE → canvasMaxSide. */
export interface Limits { readonly canvasMinSide: number; readonly canvasMaxSide: number; /* … */ readonly tokenExpiryDays: readonly number[] }
/** The same names everywhere: Rust, TypeScript, MCP, product events. */
export type ExportFormat = 'wasm' | 'gif' | 'apng' | 'sprite_sheet' | 'png_frames';
export interface ExportRequest { readonly format: ExportFormat; readonly tag?: string; readonly scale?: number }
export interface ExportedFile { readonly name: string; readonly mediaType: string; readonly bytes: Uint8Array }
export interface ExportResult { readonly files: readonly ExportedFile[]; readonly totalBytes: number }
export type Framework = 'html' | 'angular' | 'react' | 'vue';
export interface SnippetRequest {
  readonly framework: Framework; readonly src: string; readonly loader: string;
  readonly tag?: string; readonly alt: string;
}
export interface EngineError { readonly code: string; readonly params: Readonly<Record<string, unknown>> }

export interface EditorEngine {
  /* Commands: they resolve without a value, or reject with an EngineError. */
  create(options: NewAnimationOptions): Promise<void>;
  open(document: Uint8Array): Promise<void>;
  apply(operation: EditOperation): Promise<void>;
  undo(): Promise<void>;
  redo(): Promise<void>;
  markSaved(): Promise<void>;
  /* Queries. */
  render(request: RenderRequest): Promise<RenderedFrame>;
  serialize(): Promise<Uint8Array>;
  export(request: ExportRequest): Promise<ExportResult>;
  snippet(request: SnippetRequest): Promise<string>;
  /* State, published after every command; returns the unsubscribe function. */
  subscribe(listener: (state: EngineState) => void): () => void;
}
```

- `EngineStore` (`providedIn: 'root'`) subscribes once and exposes `state` as a signal, plus
  `document`, `limits` and `hasUnsavedWork` as computed signals; its commands turn a rejected
  `EngineError` into a notification with the `errors.<code>` key. It lives for the whole page, so
  moving between routes never loses the work.
- The **mock** keeps a `DocumentSummary` in TypeScript and applies the structural operations to it;
  `paintStroke` sets its points, the other pixel operations are recorded without pixels; `export`
  and `serialize` return JSON of the summary. It is test code: an ESLint `no-restricted-imports`
  rule keeps `testing/` out of production files. Until W1, `EDITOR_ENGINE` provides it.
- The **contract suite** checks, on any engine: create gives a ready state with the document;
  structural operations change the summary; undo and redo toggle it and their flags;
  `hasUnsavedWork` after a change and not after `markSaved`; `serialize` then `open` gives the same
  summary; a rejected operation leaves the state unchanged; every export format answers with files.

## U1 — Application shell

- **Root**: `<ion-app>` with a header — the name, links to the editor, the library and the settings,
  Help, and the account menu H7 fills — above an `<ion-router-outlet>`. The app bootstraps with
  `provideIonicAngular`, the router, `provideHttpClient` and `provideI18n()`.
- **Routes** (`app.routes.ts`, every page lazy):

| Path | Page | Task |
|---|---|---|
| `''` | redirect to `editor` | U1 |
| `editor`, `editor/:animationId` | the editor; the id opens a saved animation | U1, H8 |
| `settings` | language, theme, motion | U1 |
| `library`, `library/projects/:projectId` | the library | H8 |
| `sign-in`, `sign-up`, `verify-email`, `reset-password`, `reset-password/confirm`, `account` | accounts | H7 |
| `support`, `support/:requestId` | support | H9 |
| `legal/:page` | legal pages | H16 |
| `settings/tokens` | personal access tokens | A3 |
| `settings/agents` | local MCP setup, desktop only | T3 |
| `**` | not found | U1 |

- **Editor page** (`editor/editor-page.ts`): a grid of regions filled by `<lp-tool-bar>`,
  `<lp-palette-panel>`, `<lp-canvas>`, `<lp-timeline>` and `<lp-export-button>`. U1 creates each as
  an empty stub in the folder of the task that fills it (`tools/`, `palette/`, `canvas/`,
  `timeline/`, `export/`), so that the page never changes afterwards. The page's header holds the
  animation's title, editable in place (`setTitle`), and a "New" action. With no document — a
  first visit, or "New" —, a dialog asks for the title and the size, 32 × 32 by default and bounded
  by the limits, then creates the animation with the translated default layer name; "New" asks
  first when there is unsaved work.
- **EditorStore** (`editor/editor-store.ts`), the editor's UI state as signals, shared by U2 to U5:
  `tool` (`pencil`, `eraser`, `fill`, `line`, `rectangle`, `select`), `rectangleFilled`,
  `colorIndex`, `activeLayer`, `activeFrame`, `frameSelection` (a range, for tags), `zoom`, `pan`,
  `showGrid`, `onionSkin` (`enabled`, `before`, `after`), `selection` (an `Area` or none),
  `playing`. It follows the document: an active layer or frame that disappears falls back to the
  nearest one.
- **Shortcuts** (`editor/shortcuts.ts`): a registry — key, action, i18n label — that ignores keys
  typed into a text field; features register theirs; U4 shows them all in a dialog on `?`.
- **Settings**: the language list comes from `languages.json`, and switching re-renders without
  reloading; theme is system, light or dark; motion follows the system or reduces. They go through
  `PreferencesStore` (`settings/preferences-store.ts`), whose web implementation keeps them for the
  page (D37); H7 saves a signed-in person's language in the account, T2 the desktop's choices in
  its settings file.
- **Design tokens** (`projects/shared/src/styles/`): CSS custom properties `--lp-color-*`,
  `--lp-space-*`, `--lp-radius-*`, `--lp-font-*`, mapped onto Ionic's variables, with a dark set
  under `prefers-color-scheme: dark` and the manual theme; contrast AA in both. `:focus-visible`
  draws a 2-pixel outline in `--lp-color-focus`. Reduced motion shortens every animation and
  transition to nothing and turns Ionic's animations off.
- **Unsaved work**: while `hasUnsavedWork`, a `beforeunload` handler asks before the page is left or
  reloaded; nothing is kept in the browser (D37).
- **Keys**: `shell.`, `settings.`, `editor.`, `not_found.`.

**Tests**: routes resolve lazily; language switching; theme and reduced motion applied; the
`beforeunload` guard set only while there is unsaved work; axe finds no serious violation on the
shell, the settings and the not-found page.

## U2 — Canvas

`canvas/`: `lp-canvas` and its helpers.

- **Drawing**: an off-screen canvas at the animation's size receives each `RenderedFrame` through
  `ImageData`, and is drawn onto the visible canvas at an integer zoom with
  `imageSmoothingEnabled = false`, over a checkerboard. Onion skin draws the `before` previous and
  `after` next frames first, at alpha 0.3 for the nearest, halved for each step further. The grid
  shows from zoom 8, one line per pixel. At most one render per animation frame.
- **Tools** turn pointer input into operations on the active layer and frame, in pixel
  coordinates (`floor((client - origin) / zoom)`), with pointer capture:

| Tool | Pointer | Operation |
|---|---|---|
| pencil, eraser | down, move, up | the stroke's points go to `render({ preview })` on every frame; `paintStroke` on up, the eraser with index 0 |
| line, rectangle | down, move, up | previewed from the start point; applied on up; Shift constrains to 45° or a square |
| fill | click | `fill` |
| select | drag | sets `selection`; dragging inside it previews then applies `moveSelection` |

  Escape cancels a gesture in progress. Nothing is applied until the gesture ends, so one gesture
  is one undo step.
- **Zoom and pan**: zoom 1 to 64; Ctrl or ⌘ with the wheel, pinch, `+` and `-` zoom around the
  pointer; `0` fits the largest integer zoom into the view. Space with a drag, the middle button, or
  Shift with the arrows pan.
- **Keyboard**: the canvas is focusable (`role="application"`, a translated label). The arrows move
  a pixel cursor; Enter or Space applies the tool there — line, rectangle and select take a first
  press for the start and a second for the end; Alt with the arrows moves the selection one pixel.
  A polite live region announces the cursor's position and the colour index.
- **Keys**: `canvas.`.

**Tests** with the mock engine: a pencil drag sends previews then one `paintStroke` with the
dragged points; line, rectangle, fill and select send their operation; Escape sends nothing; zoom
and fit arithmetic; the grid threshold; drawing a pixel with the keyboard alone.

## U3 — Timeline

`timeline/`: `lp-timeline`.

- **Frames**: thumbnails rendered by the engine into 48-pixel boxes, refreshed at most once per
  animation frame after a change; the active frame highlighted; a duration field per frame,
  bounded by the limits; add after the active frame, duplicate, delete; reorder by drag and drop or
  Alt with ← and →; Shift with a click extends `frameSelection`.
- **Layers**: listed top layer first; add, delete, rename in place, show or hide
  (`aria-pressed`), reorder by drag and drop or Alt with ↑ and ↓.
- **Tags**: bars over their frames; "Add tag" on `frameSelection` asks a name and a loop mode;
  rename, change, delete. Names that break the rules are refused with the engine's error.
- **Playback preview**: play (`P`) exports the animation as WASM through the engine and plays it
  in a `<life-pixel>` element, from a `blob:` URL, on the tag holding the active frame or on every
  frame: the preview is the shipping player, so no timing rule is written in TypeScript. It never
  starts on its own; any change stops it. The app depends on `@life-pixel/player` as
  `"file:../player-js"`, and imports it once.
- Shortcuts: `,` and `.` for the previous and next frame, `P`, `O` for onion skin.
- **Keys**: `timeline.`.

**Tests**: each action sends its operation; reordering with the keyboard; the tag dialog; the
preview element given the export's URL and the right tag, and stopped by a change.

## U4 — Palette and tools

`palette/` (`lp-palette-panel`) and `tools/` (`lp-tool-bar`).

- **Palette**: a grid of swatches with their index; entry 0 is a checkerboard, selectable — it is
  the eraser's colour — but not editable. Selecting sets `colorIndex`; `[` and `]` move through it.
  Add and edit through a dialog: a colour input, an alpha slider 0 to 255, and a `#rrggbbaa` field
  kept in sync. Remove; reorder by drag and drop or Alt with the arrows. A full palette disables
  Add.
- **Tool bar**: one button per tool, `aria-pressed`, a tooltip with its shortcut; a filled/outlined
  switch for the rectangle; undo and redo buttons bound to `canUndo` and `canRedo`.
- **Import**: "Import image" picks a PNG and applies `importImage` on the active layer and frame,
  at the top-left corner; "Import sprite sheet" picks a PNG, asks the cell size and the frames'
  duration, and applies `importSpriteSheet` after the active frame. Files larger than the limits
  are refused before they are read.
- **Shortcuts** and the help dialog:

| Key | Action | Key | Action |
|---|---|---|---|
| `B` | pencil | `M` | select and move |
| `E` | eraser | Ctrl/⌘ `Z` | undo |
| `G` | fill | Ctrl/⌘ Shift `Z`, Ctrl `Y` | redo |
| `L` | line | `+` `-` `0` | zoom in, out, fit |
| `R`, Shift `R` | rectangle, filled | Shift `G` | grid |
| `[` `]` | previous, next colour | `?` | shortcuts |

- **Keys**: `palette.`, `tools.`, `shortcuts.`.

**Tests**: each palette action sends its operation; entry 0 cannot be edited; the dialog keeps the
colour input, the slider and the hex field in sync; both imports send their operation, and an
oversized file is refused; every shortcut, and none while typing in a field.

## U5 — Export dialog

`export/`: `lp-export-button` and the dialog it opens (also Ctrl/⌘ `E`).

- **Formats**: WASM, GIF, APNG, sprite sheet, PNG frames. When the dialog opens, the engine exports
  each in turn; every row shows the raw size and the gzip size — measured with `CompressionStream`
  —, formatted with `Intl`, and the lightest is marked. Options: the tag (all frames or one tag),
  and the scale for the raster formats, bounded by the limits.
- **Download**: through `ExportSaver` (`export/export-saver.ts`): the web implementation downloads
  each file from a `Blob`; T2 adds the desktop one. WASM gives two files: `<stem>.wasm` and
  `life-pixel.js`, the loader. After a download, the dialog calls the `EXPORT_OBSERVER` token
  (`export/export-observer.ts`) with the format and the size; its default does nothing, and H13
  provides the hosted one, which records the product event.
- **Snippets**: a framework picker, the animation's URL (`/assets/<stem>.wasm` by default), the
  loader's (`/assets/life-pixel.js`), the tag and the alternative text (the title by default); the
  code comes from `engine.snippet`, with a copy button and a translated hint on where each file
  goes (`export.snippet.hint.<framework>`).
- **Keys**: `export.`.

**Tests**: every format listed with its sizes; options re-export; downloads go through
`ExportSaver`; snippet fields reach `engine.snippet`; copying.

## W1 — Editor engine

### editor-wasm

`crates/editor-wasm`, `crate-type = ["cdylib", "rlib"]`, over `core` and `compiler`, with
`wasm-bindgen`, `serde-wasm-bindgen` and `serde_bytes`.

| File | Content |
|---|---|
| `src/engine_core.rs` | `EngineCore`: `Option<Animation>`, `History`; plain Rust methods, tested on the host |
| `src/bindings.rs` | `#[wasm_bindgen]` wrapper: one method per interface method, JSON-like values through `serde-wasm-bindgen`, bytes as `Uint8Array` |
| `src/errors.rs` | every error as `{ code, params }` |

`EngineCore` methods: `create`, `open`, `apply`, `undo`, `redo`, `mark_saved`, `state`,
`render(frame, preview)` — RGBA through `core::render`, with `edit::preview` for the preview —,
`serialize`, `export` — the WASM export yields `<stem>.wasm` and `LOADER_JS` as `life-pixel.js`,
the classic exports their files —, `snippet` through `compiler::render_snippet`, and `limits`.
Input never panics: it is validated into errors.

### Worker and adapter

`engine/wasm/`:

| File | Content |
|---|---|
| `protocol.ts` | `{ id, method, args }` requests; `{ id, ok, value }` or `{ id, ok: false, error }` responses; `{ type: 'state', state }` pushes |
| `engine.worker.ts` | loads the module, owns one `EngineCore`, answers requests in order, pushes the state after each command |
| `wasm-editor-engine.ts` | `WasmEditorEngine implements EditorEngine`: posts requests with their buffers transferred, resolves by id |
| `snapshot.ts` | the recovery snapshot |

- The worker is created with `new Worker(new URL('./engine.worker', import.meta.url), { type:
  'module' })`, and loads the module from `/engine/editor_engine_bg.wasm`.
- **Recovery**: the adapter keeps, in the page, the serialized document taken after `open`,
  `create`, every save, and every 30 seconds of changes. If the worker fails — a trap, or an error
  outside a request —, the adapter starts a new worker, reopens the snapshot, publishes `failed`
  then `ready`, and the app says how much work was lost (`engine.recovered`).
- W1 switches `EDITOR_ENGINE` to `WasmEditorEngine`; the mock stays for unit tests.

### Build

`cargo xtask build-editor` builds `life-pixel-editor-wasm` for `wasm32-unknown-unknown` in
release, checks that `wasm-bindgen --version` matches the `wasm-bindgen` of `Cargo.lock` — or fails
with the `cargo install wasm-bindgen-cli --version … --locked` command to run —, then runs
`wasm-bindgen --target web --out-name editor_engine`, writing the JavaScript glue to
`src/app/engine/wasm/generated/` and the module to `public/engine/`. Both paths are in
`.gitignore`. `frontend/tools/prebuild.mjs` runs that command; W1 puts it in front of the `start`
and `build:app` scripts, and adds `test:engine`, which builds the engine and runs W0's contract
suite against it — `test:ci` keeps to the mock, so that an interface task needs no Rust. The
prebuild is skipped when `LP_ENGINE_PREBUILT=1` and both outputs exist: the Docker build prepares
them in its Rust stage, and CI's `frontend` job builds them once and hands them to the jobs that
serve the app as an artifact.

**Tests**: `EngineCore` on the host — every method, errors as codes, preview without change,
unsaved state across save and undo —; W0's contract suite against `WasmEditorEngine` in a real
browser, through Vitest's browser mode (`test:engine`); recovery after a forced worker failure.

## U6 — End-to-end path

`frontend/e2e/editor/`, Playwright project `editor`, `npm run e2e`; the web server is `npm start`.

- **Journey**: open `/editor`; draw with the pencil, the line and the fill; add a frame and draw on
  it; set durations; tag the second frame `blink`, played once; undo and redo; open the export
  dialog and see the five formats with their sizes; download the WASM export and the loader; play
  them in a page served through `page.route`, and read back, with `getImageData`, the pixels drawn
  on each frame, and `tagend` after `blink`.
- **Keyboard**: the same drawing done with the keyboard alone.
- **Accessibility**: `@axe-core/playwright` on the editor, the export dialog, the settings and the
  shortcuts dialog, with no serious or critical violation.
- A passing U6 means M2 is done.
