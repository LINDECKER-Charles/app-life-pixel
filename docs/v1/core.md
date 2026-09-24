# Core and classic exports — K1, K2, C2

`crates/core` is the one place where a pixel rule lives (invariant 1): the model, its limits and
validation, its serialization, compositing, the editing operations and their undo. It is pure: no
I/O, no clock, no randomness, no async. `crates/compiler` turns an animation into files.

## K1 — Document model

### Layout

| Path | Content |
|---|---|
| `src/limits.rs` | every product limit, and `Limits`, their serializable summary |
| `src/error.rs` | `DocumentError`, each variant with `code()` and `params()` |
| `src/model/` | `animation.rs`, `palette.rs`, `layer.rs`, `frame.rs`, `cel.rs`, `tag.rs`, `project.rs`, `names.rs` |
| `src/serialize/` | `json.rs` (document v1), `rle.rs`, `grid.rs`, `migrate.rs` |
| `src/render/` | `composite.rs`, `rgba.rs` |

Dependencies: `serde`, `serde_json`, `base64`, `thiserror`; K2 adds `png`.

### Model

```rust
pub struct LayerId(u32);
pub struct FrameId(u32);
pub struct Point { pub x: i32, pub y: i32 }
pub struct Rgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

pub struct Animation {
    title: Name,
    width: u16,
    height: u16,
    palette: Palette,          // entry 0 is transparent, always
    layers: Vec<Layer>,        // bottom to top
    frames: Vec<Frame>,        // in play order
    cels: BTreeMap<(LayerId, FrameId), Cel>, // non-empty cels only
    tags: Vec<Tag>,
    next_id: u32,              // layers and frames take their ids from it
}
pub struct Layer { id: LayerId, name: Name, visible: bool }
pub struct Frame { id: FrameId, duration_ms: u16 }
pub struct Cel(Box<[u8]>);     // width × height palette indices, row by row
pub struct Tag { name: TagName, first: u16, last: u16, loop_mode: LoopMode } // `loop_mode` is `loop` in JSON
pub enum LoopMode { Loop, Once }  // "loop", "once"
pub struct Project { name: Name }
pub struct Name(String);       // see Names below
pub struct TagName(String);

pub struct NewAnimation {
    pub title: Name, pub width: u16, pub height: u16,
    pub layer_name: Name,               // the caller translates it
    pub frame_duration_ms: Option<u16>, // 100 by default
    pub palette: Option<Palette>,       // DEFAULT_PALETTE by default
}
```

- Fields are private; an `Animation` is only built by `Animation::new(NewAnimation)`, by parsing a
  document, or changed by K2's operations — so an `Animation` in memory is always valid. Getters
  expose everything read-only.
- Ids are local to the animation, from `next_id`; project and animation ids are UUIDs made by
  `service`.
- **Names** (titles, project and layer names): 1 to 100 characters after trimming, no control
  character. **Tag names**: 1 to 32 characters among `a`–`z`, `0`–`9`, `-`, `_`, starting with a
  letter, unique in the animation.
- `DEFAULT_PALETTE`, 16 entries: `#00000000`, `#000000ff`, `#ffffffff`, `#7f7f7fff`, `#c3c3c3ff`,
  `#880015ff`, `#ed1c24ff`, `#ff7f27ff`, `#fff200ff`, `#22b14cff`, `#00a2e8ff`, `#3f48ccff`,
  `#a349a4ff`, `#b97a57ff`, `#ffaec9ff`, `#99d9eaff`.

### Limits

Constants of `core::limits`, the only place they are written. `Limits::current()` returns them as
a serializable struct with one field per constant, named in camelCase — `CANVAS_MAX_SIDE` becomes
`canvasMaxSide` —, which the engine gives the interface, so no front-end code repeats one.

| Constant | Value | Used by |
|---|---|---|
| `CANVAS_MIN_SIDE`, `CANVAS_MAX_SIDE` | 1; 512 pixels | model |
| `MAX_FRAMES`, `MAX_LAYERS`, `MAX_TAGS` | 1,024; 64; 64 | model |
| `MAX_CEL_PIXELS` | 16,777,216 pixels in non-empty cels, all together | model |
| `MAX_PALETTE_ENTRIES` | 256, entry 0 included | model |
| `MIN_FRAME_DURATION_MS`, `MAX_FRAME_DURATION_MS`, `DEFAULT_FRAME_DURATION_MS` | 10; 65,535; 100 | model |
| `NAME_MAX_CHARS`, `TAG_NAME_MAX_CHARS` | 100; 32 — both at least 1 | model |
| `MAX_DOCUMENT_BYTES` | 33,554,432 (32 MiB) of serialized document | service, server |
| `IMPORT_MAX_SIDE`, `IMPORT_MAX_BYTES` | 4,096 pixels; 16,777,216 | import |
| `STROKE_MAX_POINTS`, `DRAW_MAX_OPERATIONS` | 10,000; 1,000 | edit, MCP |
| `HISTORY_MAX_STEPS`, `HISTORY_MAX_BYTES` | 200; 67,108,864 | history |
| `EXPORT_MIN_SCALE`, `EXPORT_MAX_SCALE`, `EXPORT_MAX_SIDE` | 1; 16; 8,192 pixels | compiler |
| `PREVIEW_MAX_SIDE`, `PREVIEW_MAX_BYTES` | 1,024 pixels; 1,048,576 | service |
| `PASSWORD_MIN_CHARS`, `PASSWORD_MAX_CHARS`, `EMAIL_MAX_CHARS` | 12; 128; 254 | accounts |
| `SUPPORT_MESSAGE_MAX_CHARS`, `SCREENSHOT_MAX_BYTES`, `SCREENSHOT_MAX_SIDE` | 5,000; 5,242,880; 4,096 pixels | support |
| `TOKEN_NAME_MAX_CHARS`, `TOKEN_EXPIRY_DAYS`, `MAX_ACTIVE_TOKENS` | 60; the list 30, 90, 365; 20 | tokens |
| `PAGE_SIZE_DEFAULT`, `PAGE_SIZE_MAX`, `MCP_PAGE_SIZE_DEFAULT`, `MCP_PAGE_SIZE_MAX` | 50; 100; 20; 50 | service, MCP |

### Validation

Parsing a document checks every invariant and fails with the first error found. Parameters are
named in camelCase, as they appear in the API's problems and in the catalogues' messages:

| Code | Params | When |
|---|---|---|
| `document.malformed` | — | not JSON, a missing or unknown field, a wrong type |
| `document.unsupported_version` | `version` | a version this build does not know |
| `document.too_large` | `maxBytes` | more than `MAX_DOCUMENT_BYTES` |
| `document.canvas_size` | `min`, `max` | a side out of `CANVAS_MIN_SIDE` to `CANVAS_MAX_SIDE` |
| `document.frame_count`, `document.layer_count`, `document.tag_count` | `max` | 0 frames or layers, or more than allowed |
| `document.pixel_budget` | `max` | beyond `MAX_CEL_PIXELS` |
| `document.palette` | `max` | no entry, more than 256, entry 0 not transparent, a colour not `#rrggbbaa` |
| `document.frame_duration` | `min`, `max` | out of `MIN_FRAME_DURATION_MS` to `MAX_FRAME_DURATION_MS` |
| `document.name` | `max` | an invalid title or name |
| `document.tag` | `name` | an invalid, duplicate or out-of-range tag |
| `document.reference` | — | a duplicate id, a cel on a missing layer or frame, `nextId` not above every id |
| `document.cel` | — | a cel of the wrong size, undecodable, or with an index at or above the palette size |

### Serialization

A document is one animation, serialized as JSON with 2-space indentation, fields in the order
below, cels sorted by layer id then frame id: the same animation always gives the same bytes.

```json
{
  "format": "life-pixel/animation",
  "version": 1,
  "title": "Mascot",
  "width": 32,
  "height": 32,
  "palette": ["#00000000", "#000000ff", "#ffffffff"],
  "layers": [{ "id": 1, "name": "Body", "visible": true }],
  "frames": [{ "id": 2, "durationMs": 100 }],
  "cels": [{ "layer": 1, "frame": 2, "rle": "AQIDBA==" }],
  "tags": [{ "name": "idle", "first": 0, "last": 0, "loop": "loop" }],
  "nextId": 3
}
```

- Names in camelCase; colours `#rrggbbaa` in lowercase; `loop` is `"loop"` or `"once"`; unknown
  fields are refused.
- A cel is `"rle"` — base64, with padding, of pairs *(LEB128 run length ≥ 1, index byte)* covering
  the cel exactly — or `"grid"`, an array of [text grid](#text-grid) rows. The writer always writes
  `rle`; `grid` is for documents a person or an agent writes, such as the samples.
- `migrate.rs` reads `format` and `version` first: `life-pixel/animation` version 1 parses as above;
  a higher version fails with `document.unsupported_version`. A future version 2 adds a
  `v1 → v2` step there, and every version stays readable.

### Text grid

The representation of `write_frame` and of the samples: one string per row, top to bottom.

- **Single mode**, when the palette has at most 62 entries: one character per pixel — `.` for 0,
  `1`–`9` for 1 to 9, `a`–`z` for 10 to 35, `A`–`Z` for 36 to 61.
- **Pair mode**, above 62 entries: two lowercase hexadecimal digits per pixel, `00` for 0.
- Every row has `width` characters in single mode, `2 × width` in pair mode, and there are
  `height` rows. `grid::format` picks the mode from the palette size; `grid::parse` from the row
  length.
- Errors: `grid.size` (`width`, `height`), `grid.character` (`row`, `column`), `grid.index`
  (`row`, `column`, `index`), rows and columns counted from 0.

### Compositing and rendering

- `render::composite(&Animation, FrameId) -> Vec<u8>`: starts with index 0 everywhere, then, for
  each visible layer from bottom to top, copies every non-zero index of its cel. Hidden layers are
  skipped. `composite_with` takes one replacement cel, for previews.
- `render::rgba(&Animation, FrameId) -> Vec<u8>`: the composite through the palette, 4 bytes per
  pixel, alpha not premultiplied. The editor, the exports and the MCP previews all go through it:
  what the editor shows is what ships.

**Tests**: a new animation, and every invariant of the validation table, each failing with its
code; JSON round trip, byte for byte; a document with grid cels reads as the same animation as its
`rle` twin; single and pair grid modes, both ways, and each grid error; a version 2 document
refused; compositing with hidden layers and with index 0 over a lower layer; `Limits::current()`
serialized.

## K2 — Editing operations

### Operations

`core::edit::Operation`, serialized with `#[serde(tag = "kind", rename_all = "camelCase",
rename_all_fields = "camelCase")]` — the shape the engine interface of
[editor.md](editor.md#w0--engine-interface) mirrors field for field.

| Operation | Fields | Rules |
|---|---|---|
| `paintStroke` | `layer`, `frame`, `points`, `index` | the pencil, or the eraser with index 0; each point, and a line between consecutive points; pixels outside the canvas ignored; at most `STROKE_MAX_POINTS` points |
| `fill` | `layer`, `frame`, `at`, `index` | 4-connected flood fill of the layer's own cel, not of the composite; `at` outside the canvas: `edit.out_of_canvas`; same index: no change |
| `line` | `layer`, `frame`, `from`, `to`, `index` | Bresenham, both ends included, clipped |
| `rectangle` | `layer`, `frame`, `from`, `to`, `index`, `filled` | corners in any order, included; outline one pixel wide |
| `moveSelection` | `layer`, `frame`, `area`, `offset` | `area` (`x`, `y`, `width`, `height`) clipped to the canvas is cleared to 0, then its pixels are pasted at `offset`; pasted index-0 pixels leave the destination as it was; pixels leaving the canvas are lost |
| `setPaletteEntry` | `index`, `color` | `index ≥ 1` |
| `addPaletteEntry` | `color` | appended; `edit.palette_full` at 256 |
| `removePaletteEntry` | `index` | `index ≥ 1`; its pixels become 0, higher indices move down by one |
| `movePaletteEntry` | `from`, `to` | both ≥ 1; every cel remapped |
| `addLayer` | `position`, `name` | 0 is the bottom |
| `deleteLayer` | `layer` | its cels go too; the only layer: `edit.last_layer` |
| `moveLayer`, `renameLayer`, `setLayerVisibility` | `layer` and `position`, `name` or `visible` | |
| `addFrame` | `position`, `durationMs` | empty cels |
| `duplicateFrame` | `frame` | the copy goes right after it, cels included |
| `deleteFrame` | `frame` | the only frame: `edit.last_frame` |
| `moveFrame` | `frame`, `position` | tags keep their positions |
| `setFrameDuration` | `frame`, `durationMs` | |
| `addTag`, `updateTag`, `deleteTag` | `tag`; `name` and `tag`; `name` | a tag is found by its name |
| `replaceTags` | `tags` | every tag at once, as MCP's `set_tags` needs |
| `setTitle` | `title` | |
| `importImage` | `layer`, `frame`, `png`, `at` | see [Import](#import) |
| `importSpriteSheet` | `layer`, `position`, `png`, `cellWidth`, `cellHeight`, `durationMs` | see [Import](#import) |

- Inserting a frame at position *p* moves every tag starting at or after *p* one position later,
  and extends a tag that spans *p*; deleting one mirrors it, and deletes a tag of that frame alone.
- Every reference is checked first: `edit.layer_not_found`, `edit.frame_not_found`,
  `edit.tag_not_found`, `document.palette` for an index at or above the palette size; the other
  rules of the model hold after every operation, or it fails with the model's code, unchanged.
- `edit::preview(&Animation, &Operation) -> Result<Option<(LayerId, FrameId, Cel)>, EditError>`
  returns, for a pixel operation, the cel it would produce, without changing the animation: the
  canvas draws strokes, lines and rectangles live with it.

### History

`core::edit::History` holds the undo and redo stacks.

```rust
impl History {
    pub fn apply(&mut self, animation: &mut Animation, operation: Operation) -> Result<(), EditError>;
    pub fn undo(&mut self, animation: &mut Animation) -> bool;
    pub fn redo(&mut self, animation: &mut Animation) -> bool;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn mark_saved(&mut self);
    pub fn is_saved(&self) -> bool; // true when the animation is back to the marked state
}
```

- Applying an operation returns its inverse — the cels, palette, layers, frames or tags it
  replaced — kept with it; applying clears the redo stack.
- The oldest steps are dropped beyond `HISTORY_MAX_STEPS` or `HISTORY_MAX_BYTES` of inverses.
- A failed operation leaves both the animation and the history as they were.

### Import

- A PNG is read through the `png` crate: its size is checked against `IMPORT_MAX_BYTES` before
  reading, its dimensions against `IMPORT_MAX_SIDE` right after the header — before any pixel is
  decoded —, then it is expanded to RGBA 8 bits. Failures: `import.image_too_large` (`maxSide`,
  `maxBytes`), `import.image_malformed`.
- Each pixel with an alpha below 128 maps to 0; the others to the nearest palette entry of alpha at
  least 128 by squared distance on red, green and blue, the lowest index winning ties.
- `importImage` pastes at `at`, clipped; pixels mapped to 0 leave the cel as it was.
- `importSpriteSheet` cuts the image into `cellWidth × cellHeight` cells, row by row, drops the
  fully transparent cells at the end, and inserts one frame per cell at `position`, its pixels on
  `layer`. A grid that does not fit: `import.sheet_grid`.

**Tests**: every operation and its undo, restoring the document byte for byte; the frame and tag
rules; each error, leaving the document unchanged; `preview` equal to what `apply` produces;
history limits and `is_saved` across undo and redo; flood fill against a spiral; Bresenham in all
octants; import of paletted, grey, 16-bit and transparent PNGs, and a PNG whose header claims
100,000 × 100,000 pixels refused without allocating. Golden images, `crates/core/tests/golden/`:
RGBA renders after scripted sequences, compared pixel for pixel; `LP_UPDATE_GOLDEN=1` rewrites
them, for review in the diff.

## C2 — Classic exports

`crates/compiler`, created here: `src/lib.rs`, `src/naming.rs`, `src/classic/` with `gif.rs`,
`apng.rs`, `sprite_sheet.rs`, `png_frames.rs`, `scale.rs`. Dependencies: `core`, `gif`, `png`,
`zip` with `default-features = false` — pure Rust, so that `editor-wasm` still builds for
`wasm32` —, `serde_json`, `thiserror`.

```rust
/// Serialized "wasm", "gif", "apng", "sprite_sheet", "png_frames": the names used everywhere.
pub enum ExportFormat { Wasm, Gif, Apng, SpriteSheet, PngFrames }
pub struct ClassicOptions { pub tag: Option<String>, pub scale: u8 }
pub struct ExportFile { pub name: String, pub media_type: &'static str, pub bytes: Vec<u8> }
pub fn export_gif(animation: &Animation, options: &ClassicOptions) -> Result<ExportFile, ExportError>;
pub fn export_apng(animation: &Animation, options: &ClassicOptions) -> Result<ExportFile, ExportError>;
pub fn export_sprite_sheet(animation: &Animation, options: &ClassicOptions)
    -> Result<Vec<ExportFile>, ExportError>; // the PNG, then the JSON
pub fn export_png_frames(animation: &Animation, options: &ClassicOptions)
    -> Result<ExportFile, ExportError>; // a zip
pub fn file_stem(title: &str) -> String;
```

- **Range and scale**: `tag` limits the export to that tag's frames and takes its loop mode;
  without it, every frame, looping. `scale` repeats each pixel `scale × scale` times;
  `EXPORT_MIN_SCALE`, `EXPORT_MAX_SCALE` and `EXPORT_MAX_SIDE` bound it (`export.scale`,
  `export.too_large`).
  `export.tag_not_found` for an unknown tag.
- **Names**: `file_stem` lowercases the title, keeps ASCII letters and digits, joins the rest with
  single `-`, and falls back to `animation`: `mascot.gif`, `mascot.png` and `mascot.json`,
  `mascot-frames.zip` holding `mascot-000.png` and on (as many digits as the frame count needs).
- **GIF**: global palette from the animation's colours (partial alpha becomes opaque, as export.md
  warns), transparent index 0, every frame full size with disposal "restore to background",
  delay `max(2, round(ms / 10))` hundredths of a second; the NETSCAPE loop extension with an
  infinite count for a looping range, none for a range played once.
- **APNG**: indexed colour with `PLTE` and `tRNS` — full alpha kept —, one full frame per frame,
  delay `ms/1000`, plays `0` for a looping range, `1` for a range played once.
- **Sprite sheet**: frames in a grid of `ceil(√n)` columns, row by row, no padding, as an indexed
  PNG; the JSON follows Aseprite's "array" layout, which most engines import:

```json
{
  "frames": [
    { "filename": "mascot 0", "frame": { "x": 0, "y": 0, "w": 32, "h": 32 }, "rotated": false,
      "trimmed": false, "spriteSourceSize": { "x": 0, "y": 0, "w": 32, "h": 32 },
      "sourceSize": { "w": 32, "h": 32 }, "duration": 100 }
  ],
  "meta": {
    "app": "Life Pixel", "version": "1", "image": "mascot.png", "format": "RGBA8888",
    "size": { "w": 128, "h": 96 }, "scale": "1",
    "frameTags": [{ "name": "jump", "from": 4, "to": 7, "direction": "forward", "repeat": "1" }]
  }
}
```

  `repeat` appears only on tags played once.
- **PNG frames**: a zip of indexed PNGs, stored uncompressed — PNGs already are — in frame order,
  with the zip's fixed date of 1980-01-01, so that it is deterministic like every other export.

**Tests**: golden files for GIF, APNG, sprite sheet and zip of two small animations, byte for byte;
each export decoded back — `gif` and `png` crates, the JSON parsed — shows the frames, delays,
loop modes and transparency expected; scale and tag options; each error.
