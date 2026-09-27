//! The v1 fixture of `crates/format`: the payload, and what its expected JSON says it holds.

use std::fs;
use std::path::Path;

use anyhow::Context;
use serde_json::Value;

/// The payload, relative to the workspace root.
const PAYLOAD: &str = "crates/format/tests/fixtures/v1/sample.lpix";

/// Its header, palette, title, tags and frames, relative to the workspace root.
const EXPECTED: &str = "crates/format/tests/fixtures/v1/sample.expected.json";

/// A tag as the fixture expects it.
pub struct Tag {
    pub name: String,
    pub first: u32,
    pub last: u32,
    pub is_looping: bool,
}

/// A frame as the fixture expects it: its duration, and its pixels in RGBA through the palette.
pub struct Frame {
    pub duration_ms: u32,
    pub rgba: Vec<u8>,
}

/// The fixture: the payload, and what a player must show when it plays it.
pub struct Fixture {
    pub payload: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub tags: Vec<Tag>,
    pub frames: Vec<Frame>,
}

impl Fixture {
    /// Reads the fixture from the workspace at `root`.
    pub fn read(root: &Path) -> anyhow::Result<Self> {
        let payload = fs::read(root.join(PAYLOAD)).with_context(|| format!("reading {PAYLOAD}"))?;
        let text =
            fs::read_to_string(root.join(EXPECTED)).with_context(|| format!("reading {EXPECTED}"))?;
        let expected: Value =
            serde_json::from_str(&text).with_context(|| format!("parsing {EXPECTED}"))?;
        let palette = palette(&expected["palette"]).context("reading the expected palette")?;
        Ok(Self {
            payload,
            width: number(&expected["width"]).context("reading the expected width")?,
            height: number(&expected["height"]).context("reading the expected height")?,
            title: text_of(&expected["title"]).context("reading the expected title")?,
            tags: items(&expected["tags"], tag).context("reading the expected tags")?,
            frames: items(&expected["frames"], |value| frame(value, &palette))
                .context("reading the expected frames")?,
        })
    }
}

fn tag(value: &Value) -> Option<Tag> {
    Some(Tag {
        name: text_of(&value["name"])?,
        first: number(&value["first"])?,
        last: number(&value["last"])?,
        is_looping: value["loop_mode"].as_str()? == "loop",
    })
}

fn frame(value: &Value, palette: &[Vec<u8>]) -> Option<Frame> {
    let indices = value["indices"].as_array()?.iter();
    let colours = indices.map(|index| palette.get(usize::try_from(index.as_u64()?).ok()?));
    let colours = colours.collect::<Option<Vec<_>>>()?;
    let rgba = colours.into_iter().flatten().copied().collect();
    Some(Frame {
        duration_ms: number(&value["duration_ms"])?,
        rgba,
    })
}

/// The palette's entries, each as the 4 bytes of RGBA a framebuffer holds.
fn palette(value: &Value) -> Option<Vec<Vec<u8>>> {
    items(value, |entry| items(entry, |channel| u8::try_from(channel.as_u64()?).ok()))
}

fn items<T>(value: &Value, item: impl Fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    value.as_array()?.iter().map(item).collect()
}

fn number(value: &Value) -> Option<u32> {
    u32::try_from(value.as_u64()?).ok()
}

fn text_of(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}
