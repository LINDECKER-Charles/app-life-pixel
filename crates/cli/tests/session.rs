//! The scripted session of `docs/v1/mcp-cli.md`'s "A5 — MCP end to end", shared between
//! `crates/cli/tests/mcp_stdio.rs` (stdio) and `crates/server/tests/mcp_http.rs` (HTTP, included
//! by path): both connect through `().serve(transport)`, which gives the same client type
//! whichever transport carries it, so the whole script — everything but `export` and the
//! transport-specific setup — is written once here.
//!
//! `export` differs by transport (a `directory` locally, signed links hosted), so each test calls
//! it and reads its bytes on its own, then hands them to [`check_wasm`].

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]
#![allow(dead_code, reason = "shared by two test binaries, each using a subset")]

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use life_pixel_core::edit::{Operation, apply};
use life_pixel_core::serialize::grid;
use life_pixel_core::{
    Animation, FrameId, LayerId, Name, NewAnimation, Palette, Point, Rgba, render,
};
use rmcp::RoleClient;
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::service::RunningService;
use serde_json::{Map, Value, json};

/// The animation's canvas side, in pixels.
pub const SIDE: u16 = 16;
/// `set_palette`'s colours: entry 0 transparent (never applied), five opaque ones.
pub const PALETTE: [&str; 6] = [
    "#00000000",
    "#ff0000ff",
    "#00ff00ff",
    "#0000ffff",
    "#ffff00ff",
    "#2a2a2aff",
];
/// The single-character grid alphabet for the six palette entries above, index first.
const CHARS: [char; 6] = ['.', '1', '2', '3', '4', '5'];
/// The tag `set_tags` adds, on frame 1, played once.
pub const TAG_NAME: &str = "blink";
/// The name of the new animation's only layer.
const LAYER_NAME: &str = "Layer 1";
/// The animation's title.
const TITLE: &str = "Mascot";

/// What playing the session leaves for the caller: the animation's id, ready for `export` and
/// `get_embed_snippet`, and frame 0's final composite, ready for [`check_wasm`].
pub struct Played {
    /// The animation's id, as the protocol returns it.
    pub animation_id: Value,
    /// Frame 0's composite, as a text grid, once `write_frame` and `draw` are done.
    pub frame0_rows: Vec<String>,
}

/// Calls `tool` with `arguments`, failing the test if the call errors: the structured result.
pub async fn call(client: &RunningService<RoleClient, ()>, tool: &str, arguments: Value) -> Value {
    let arguments = arguments.as_object().cloned().unwrap_or_default();
    let request = CallToolRequestParams::new(tool.to_owned()).with_arguments(arguments);
    let result = client.call_tool(request).await.unwrap();
    assert_eq!(result.is_error, Some(false), "{tool}: {result:?}");
    result.structured_content.clone().unwrap_or(Value::Null)
}

/// Plays steps 1 to 6 of the session on `client`: creates the animation, sets its palette, writes
/// and draws frame 0, adds frame 1 and writes it, tags it, checks the previews decode to their
/// announced sizes, and checks `get_animation` returns frame 1's grid as written.
pub async fn play(client: &RunningService<RoleClient, ()>) -> Played {
    let id = create_and_paint_frame0(client).await;
    let grid1 = add_and_tag_frame1(client, &id).await;

    check_preview_size(client, &id, Some(0)).await;
    check_preview_size(client, &id, None).await;

    let described = call(
        client,
        "get_animation",
        json!({ "id": id, "include_pixels": true }),
    )
    .await;
    let grids = described["grids"].as_array().unwrap();
    let frame0_rows = grid_rows(grids, 0);
    assert_eq!(
        grid_rows(grids, 1),
        grid1,
        "get_animation returns frame 1's grid as write_frame wrote it"
    );

    Played {
        animation_id: id,
        frame0_rows,
    }
}

/// Creates the animation, sets its palette, writes frame 0, then draws a line and a fill on it:
/// its id.
async fn create_and_paint_frame0(client: &RunningService<RoleClient, ()>) -> Value {
    let id = create_animation(client).await;
    paint_frame0(client, &id).await;
    id
}

/// Creates the animation and sets its palette: its id.
async fn create_animation(client: &RunningService<RoleClient, ()>) -> Value {
    let created = call(
        client,
        "create_animation",
        json!({
            "title": TITLE,
            "width": SIDE,
            "height": SIDE,
            "project_name": "Session",
        }),
    )
    .await;
    let id = created["id"].clone();

    call(
        client,
        "set_palette",
        json!({ "id": id, "palette": PALETTE }),
    )
    .await;

    id
}

/// Writes frame 0, then draws a line and a fill on it.
async fn paint_frame0(client: &RunningService<RoleClient, ()>, id: &Value) {
    call(
        client,
        "write_frame",
        json!({ "id": id, "frame": 0, "grid": solid_grid(1) }),
    )
    .await;
    call(
        client,
        "draw",
        json!({
            "id": id,
            "frame": 0,
            "operations": [
                { "op": "line", "from": [0, 0], "to": [15, 15], "index": 2 },
                { "op": "fill", "x": 0, "y": 0, "index": 3 },
            ],
        }),
    )
    .await;
}

/// Adds frame 1, sets both frames' durations, writes frame 1's grid, then tags it "blink",
/// played once: the grid it wrote.
async fn add_and_tag_frame1(client: &RunningService<RoleClient, ()>, id: &Value) -> Vec<String> {
    add_frame1(client, id).await;
    let grid1 = checkerboard();
    call(
        client,
        "write_frame",
        json!({ "id": id, "frame": 1, "grid": grid1 }),
    )
    .await;

    call(
        client,
        "set_tags",
        json!({
            "id": id,
            "tags": [{ "name": TAG_NAME, "first": 1, "last": 1, "loop": "once" }],
        }),
    )
    .await;

    grid1
}

/// Adds frame 1 and sets both frames' durations.
async fn add_frame1(client: &RunningService<RoleClient, ()>, id: &Value) {
    call(
        client,
        "edit_frames",
        json!({
            "id": id,
            "edits": [
                { "op": "add", "position": 1 },
                { "op": "set_duration", "frame": 0, "duration_ms": 80 },
                { "op": "set_duration", "frame": 1, "duration_ms": 200 },
            ],
        }),
    )
    .await;
}

/// `get_embed_snippet` for a plain HTML page: its code names the `<life-pixel>` element.
pub async fn check_embed_snippet(client: &RunningService<RoleClient, ()>, id: &Value) {
    let snippet = call(
        client,
        "get_embed_snippet",
        json!({ "id": id, "framework": "html" }),
    )
    .await;
    let code = snippet["code"].as_str().unwrap();
    assert!(code.contains("<life-pixel"), "{code}");
}

/// Loads `bytes` — an `export` of the session's animation in the `wasm` format — in `wasmi`
/// through ABI v1: `load` succeeds, the only tag is `blink`, and frame 0's framebuffer matches
/// `core::render::rgba` for the same pixels `write_frame` and `draw` left it with.
///
/// Runs on a dedicated thread with a larger stack: `wasmi`'s validation and instantiation need
/// more than the default stack of a `#[tokio::test]`'s thread.
pub fn check_wasm(bytes: &[u8], frame0_rows: &[String]) {
    let bytes = bytes.to_vec();
    let frame0_rows = frame0_rows.to_vec();
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let expected = expected_frame0(&frame0_rows);

            let mut player = wasm_check::MiniPlayer::load(&bytes);
            assert_eq!(player.tag_names(), vec![TAG_NAME.to_owned()]);
            let selected = player.call("set_tag", (wasm_check::WHOLE_ANIMATION,));
            assert_eq!(selected, wasm_check::DONE);
            assert_eq!(player.call("frame_index", ()), 0);
            assert_eq!(
                player.frame(),
                expected,
                "frame 0's framebuffer differs from core::render::rgba"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The RGBA `core::render::rgba` renders for frame 0 of a fresh animation given the same title,
/// size and palette as the session's, its only layer painted pixel by pixel with `frame0_rows` —
/// the composite `get_animation` returned once `write_frame` and `draw` were done.
fn expected_frame0(frame0_rows: &[String]) -> Vec<u8> {
    let palette: Vec<Rgba> = PALETTE.iter().map(|hex| hex.parse().unwrap()).collect();
    let mut animation = Animation::new(NewAnimation {
        title: Name::new(TITLE).unwrap(),
        width: SIDE,
        height: SIDE,
        layer_name: Name::new(LAYER_NAME).unwrap(),
        frame_duration_ms: None,
        palette: Some(Palette::new(palette).unwrap()),
    })
    .unwrap();
    // `Animation::new` gives its layer id 1 and its frame id 2.
    let (layer, frame) = (LayerId::new(1), FrameId::new(2));
    let cel = grid::parse(frame0_rows, animation.cel_shape()).unwrap();
    for (position, &index) in cel.indices().iter().enumerate() {
        let x = i32::try_from(position % usize::from(SIDE)).unwrap();
        let y = i32::try_from(position / usize::from(SIDE)).unwrap();
        let operation = Operation::PaintStroke {
            layer,
            frame,
            points: vec![Point { x, y }],
            index: u32::from(index),
        };
        apply(&mut animation, &operation).unwrap();
    }
    render::rgba(&animation, frame)
}

/// `render_preview` of `frame` (the contact sheet when `None`): checks the PNG decodes to the
/// size the text content announces.
async fn check_preview_size(
    client: &RunningService<RoleClient, ()>,
    id: &Value,
    frame: Option<u16>,
) {
    let mut arguments = Map::new();
    arguments.insert("id".to_owned(), id.clone());
    if let Some(frame) = frame {
        arguments.insert("frame".to_owned(), json!(frame));
    }
    let request = CallToolRequestParams::new("render_preview").with_arguments(arguments);
    let result = client.call_tool(request).await.unwrap();
    assert_eq!(result.is_error, Some(false), "render_preview: {result:?}");
    let (bytes, layout) = preview_image_and_layout(&result.content);
    let announced = (
        layout["width"].as_u64().unwrap(),
        layout["height"].as_u64().unwrap(),
    );
    assert_eq!(u64_size(png_size(&bytes)), announced);
}

/// `render_preview`'s content blocks, split into the image's decoded bytes and the layout its
/// text content announces.
fn preview_image_and_layout(content: &[ContentBlock]) -> (Vec<u8>, Value) {
    let image = content
        .iter()
        .find_map(|block| match block {
            ContentBlock::Image(image) => Some(image.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("render_preview returns an image content: {content:?}"));
    let layout = content
        .iter()
        .find_map(|block| match block {
            ContentBlock::Text(text) => Some(text.text.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("render_preview returns a text content: {content:?}"));
    let bytes = STANDARD.decode(image.data).unwrap();
    (bytes, serde_json::from_str(&layout).unwrap())
}

fn u64_size((width, height): (u32, u32)) -> (u64, u64) {
    (u64::from(width), u64::from(height))
}

/// The PNG `bytes`' size, read from its header only.
fn png_size(bytes: &[u8]) -> (u32, u32) {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let header = decoder.read_header_info().unwrap();
    (header.width, header.height)
}

/// A grid of `SIDE` rows, every pixel painted `index`.
fn solid_grid(index: usize) -> Vec<String> {
    let row = CHARS[index].to_string().repeat(usize::from(SIDE));
    vec![row; usize::from(SIDE)]
}

/// A checkerboard of index 4 and index 0 (transparent), `SIDE` rows.
fn checkerboard() -> Vec<String> {
    (0..SIDE)
        .map(|y| {
            (0..SIDE)
                .map(|x| if (x + y) % 2 == 0 { CHARS[4] } else { CHARS[0] })
                .collect::<String>()
        })
        .collect()
}

/// The rows of the grid at `frame` in `grids`, as `get_animation` returned them.
fn grid_rows(grids: &[Value], frame: u16) -> Vec<String> {
    let entry = grids
        .iter()
        .find(|grid| grid["frame"].as_u64() == Some(u64::from(frame)))
        .unwrap_or_else(|| panic!("no grid for frame {frame} in {grids:?}"));
    entry["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row.as_str().unwrap().to_owned())
        .collect()
}

/// A minimal player of ABI v1, just enough for [`check_wasm`]: `crates/compiler/tests` already
/// plays every frame and range of every fixture, so this only loads an export and reads what
/// `check_wasm` checks.
mod wasm_check {
    use wasmi::{Engine, Instance, Linker, Memory, Module, Store, WasmParams};

    /// The status of a call that completed.
    pub const DONE: u32 = 0;
    /// `set_tag`'s index for the whole animation.
    pub const WHOLE_ANIMATION: u32 = u32::MAX;

    /// The custom section that holds the payload.
    const SECTION_NAME: &str = "life-pixel";
    /// The ABI the player implements.
    const ABI_VERSION: u32 = 1;
    /// The bytes of one RGBA pixel.
    const RGBA_BYTES: usize = 4;

    /// An export's module, instantiated with its payload loaded.
    pub struct MiniPlayer {
        store: Store<()>,
        instance: Instance,
        memory: Memory,
    }

    impl MiniPlayer {
        /// Compiles `export`, checks that it imports nothing, instantiates it, then takes the
        /// loader's steps: `abi_version`, `alloc`, a copy of its `life-pixel` section, `load`.
        pub fn load(export: &[u8]) -> Self {
            let engine = Engine::default();
            let module = Module::new(&engine, export).unwrap();
            assert_eq!(module.imports().count(), 0, "an export imports nothing");
            let payload = payload(&module);
            let mut store = Store::new(&engine, ());
            let linker = Linker::new(&engine);
            let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
            let memory = instance.get_memory(&store, "memory").unwrap();
            let mut player = Self {
                store,
                instance,
                memory,
            };
            assert_eq!(player.call("abi_version", ()), ABI_VERSION);
            let len = u32::try_from(payload.len()).unwrap();
            let pointer = player.call("alloc", (len,));
            assert_ne!(pointer, 0, "alloc({len}) returned 0");
            let address = usize::try_from(pointer).unwrap();
            player
                .memory
                .write(&mut player.store, address, &payload)
                .unwrap();
            assert_eq!(player.call("load", (pointer, len)), DONE, "load's status");
            player
        }

        /// Calls the export `name`: every ABI v1 export returns a `u32`.
        pub fn call<Params: WasmParams>(&mut self, name: &str, params: Params) -> u32 {
            let export = self
                .instance
                .get_typed_func::<Params, u32>(&self.store, name);
            let export = export.unwrap_or_else(|error| panic!("the export `{name}`: {error}"));
            export.call(&mut self.store, params).unwrap()
        }

        /// The framebuffer: `width × height` RGBA pixels.
        pub fn frame(&mut self) -> Vec<u8> {
            let pixels = self.call("width", ()) * self.call("height", ());
            let len = usize::try_from(pixels).unwrap() * RGBA_BYTES;
            let pointer = self.call("frame_ptr", ());
            self.read(pointer, len)
        }

        /// The name of every tag, read through `tag_count`, `tag_name_ptr` and `tag_name_len`.
        pub fn tag_names(&mut self) -> Vec<String> {
            let tag_count = self.call("tag_count", ());
            (0..tag_count)
                .map(|index| {
                    let pointer = self.call("tag_name_ptr", (index,));
                    let len = self.call("tag_name_len", (index,));
                    self.text(pointer, len)
                })
                .collect()
        }

        fn text(&self, pointer: u32, len: u32) -> String {
            String::from_utf8(self.read(pointer, usize::try_from(len).unwrap())).unwrap()
        }

        fn read(&self, pointer: u32, len: usize) -> Vec<u8> {
            let mut bytes = vec![0; len];
            let address = usize::try_from(pointer).unwrap();
            self.memory.read(&self.store, address, &mut bytes).unwrap();
            bytes
        }
    }

    /// The payload: the content of the module's only `life-pixel` section, as the loader reads
    /// it.
    fn payload(module: &Module) -> Vec<u8> {
        let sections: Vec<&[u8]> = module
            .custom_sections()
            .filter(|section| section.name() == SECTION_NAME)
            .map(|section| section.data())
            .collect();
        assert_eq!(
            sections.len(),
            1,
            "an export has one `{SECTION_NAME}` section"
        );
        sections[0].to_vec()
    }
}
