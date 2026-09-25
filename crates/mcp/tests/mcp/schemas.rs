//! What `tools/list` shows: every tool of mcp.md, in English, with `snake_case` arguments and the
//! bounds of `core::limits`.

use life_pixel_core::limits::{
    CANVAS_MAX_SIDE, CANVAS_MIN_SIDE, DRAW_MAX_OPERATIONS, EXPORT_MAX_SCALE, EXPORT_MIN_SCALE,
    MAX_FRAME_DURATION_MS, MAX_FRAMES, MAX_PALETTE_ENTRIES, MAX_TAGS, MCP_PAGE_SIZE_DEFAULT,
    MCP_PAGE_SIZE_MAX, MIN_FRAME_DURATION_MS, NAME_MAX_CHARS, TAG_NAME_MAX_CHARS,
};
use rmcp::model::Tool;
use serde_json::{Value, json};

use super::session::Session;

/// Every tool of mcp.md, in its order.
const TOOL_NAMES: [&str; 11] = [
    "list_animations",
    "create_animation",
    "get_animation",
    "set_palette",
    "write_frame",
    "draw",
    "edit_frames",
    "set_tags",
    "render_preview",
    "export",
    "get_embed_snippet",
];

async fn tools() -> Vec<Tool> {
    Session::start()
        .await
        .client
        .list_all_tools()
        .await
        .unwrap()
}

fn tool<'a>(tools: &'a [Tool], name: &str) -> &'a Tool {
    tools.iter().find(|tool| tool.name == name).unwrap()
}

/// The schema of the argument `argument` of the tool `name`.
fn argument(tools: &[Tool], (name, argument): (&str, &str)) -> Value {
    tool(tools, name).input_schema["properties"][argument].clone()
}

fn bounds(schema: &Value) -> (Value, Value) {
    (schema["minimum"].clone(), schema["maximum"].clone())
}

#[tokio::test]
async fn every_tool_is_listed_with_a_description_and_an_object_schema() {
    let tools = tools().await;
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(names, TOOL_NAMES);
    for tool in &tools {
        assert!(
            tool.description
                .as_ref()
                .is_some_and(|text| text.len() > 40)
        );
        assert_eq!(tool.input_schema["type"], "object", "{}", tool.name);
        assert!(tool.input_schema.get("title").is_none(), "{}", tool.name);
    }
}

#[tokio::test]
async fn arguments_are_snake_case() {
    for tool in tools().await {
        let properties = tool.input_schema["properties"].as_object().unwrap().clone();
        for name in properties.keys() {
            let is_snake = name.chars().all(|c| c.is_ascii_lowercase() || c == '_');
            assert!(is_snake, "{}: {name}", tool.name);
        }
    }
}

#[tokio::test]
async fn read_tools_are_hinted_read_only() {
    let tools = tools().await;
    for (name, is_read_only) in [
        ("list_animations", true),
        ("draw", false),
        ("export", false),
    ] {
        let hint = tool(&tools, name)
            .annotations
            .as_ref()
            .unwrap()
            .read_only_hint;
        assert_eq!(hint, Some(is_read_only), "{name}");
    }
}

#[tokio::test]
async fn library_bounds_are_those_of_core() {
    let tools = tools().await;
    let limit = argument(&tools, ("list_animations", "limit"));
    assert_eq!(bounds(&limit), (json!(1), json!(MCP_PAGE_SIZE_MAX)));
    assert_eq!(limit["default"], json!(MCP_PAGE_SIZE_DEFAULT));
    let width = argument(&tools, ("create_animation", "width"));
    assert_eq!(
        bounds(&width),
        (json!(CANVAS_MIN_SIDE), json!(CANVAS_MAX_SIDE))
    );
    let duration = argument(&tools, ("create_animation", "frame_duration_ms"));
    let expected = (json!(MIN_FRAME_DURATION_MS), json!(MAX_FRAME_DURATION_MS));
    assert_eq!(bounds(&duration), expected);
    let title = argument(&tools, ("create_animation", "title"));
    assert_eq!(title["maxLength"], json!(NAME_MAX_CHARS));
}

#[tokio::test]
async fn editing_bounds_are_those_of_core() {
    let tools = tools().await;
    let palette = argument(&tools, ("set_palette", "palette"));
    assert_eq!(palette["maxItems"], json!(MAX_PALETTE_ENTRIES));
    let frame = argument(&tools, ("write_frame", "frame"));
    assert_eq!(frame["maximum"], json!(MAX_FRAMES - 1));
    let rows = argument(&tools, ("write_frame", "grid"));
    assert_eq!(rows["maxItems"], json!(CANVAS_MAX_SIDE));
    let operations = argument(&tools, ("draw", "operations"));
    assert_eq!(operations["maxItems"], json!(DRAW_MAX_OPERATIONS));
    let tags = argument(&tools, ("set_tags", "tags"));
    assert_eq!(tags["maxItems"], json!(MAX_TAGS));
    let tag_name = &tool(&tools, "set_tags").input_schema["$defs"]["TagArgument"];
    assert_eq!(
        tag_name["properties"]["name"]["maxLength"],
        json!(TAG_NAME_MAX_CHARS)
    );
    for name in ["render_preview", "export"] {
        let scale = argument(&tools, (name, "scale"));
        assert_eq!(
            bounds(&scale),
            (json!(EXPORT_MIN_SCALE), json!(EXPORT_MAX_SCALE))
        );
    }
}

#[tokio::test]
async fn palette_indices_are_bounded_by_the_palette() {
    let tools = tools().await;
    let definitions = &tool(&tools, "draw").input_schema["$defs"]["DrawOperationArgument"];
    for variant in definitions["oneOf"].as_array().unwrap() {
        let index = &variant["properties"]["index"];
        assert_eq!(index["maximum"], json!(MAX_PALETTE_ENTRIES - 1));
    }
}

#[tokio::test]
async fn the_grid_tools_state_the_alphabet() {
    let tools = tools().await;
    for name in ["write_frame", "get_animation"] {
        let description = tool(&tools, name).description.clone().unwrap();
        for symbol in ["'.' for 0", "'1'-'9'", "'a'-'z'", "'A'-'Z'", "hexadecimal"] {
            assert!(description.contains(symbol), "{name}: {symbol}");
        }
    }
}

#[tokio::test]
async fn the_export_schema_joins_the_transport_s_own_arguments() {
    let tools = tools().await;
    let schema = &tool(&tools, "export").input_schema;
    assert_eq!(schema["properties"]["destination"]["type"], "string");
    let formats = &schema["$defs"]["ExportFormatArgument"]["oneOf"];
    let formats: Vec<&Value> = formats
        .as_array()
        .unwrap()
        .iter()
        .map(|f| &f["const"])
        .collect();
    let expected = ["wasm", "gif", "apng", "sprite_sheet", "png_frames"];
    assert_eq!(
        formats,
        expected.map(Value::from).iter().collect::<Vec<_>>()
    );
    assert_eq!(schema["required"], json!(["id", "format", "destination"]));
}
