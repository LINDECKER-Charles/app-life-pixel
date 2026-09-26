//! The command line, as `docs/v1/mcp-cli.md`'s "A4 — CLI" defines it.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use life_pixel_compiler::ExportFormat;
use uuid::Uuid;

/// The Life Pixel CLI: `mcp`, `list`, `export`.
#[derive(Parser)]
#[command(name = "life-pixel", version, about)]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// `mcp`, `list` or `export`.
#[derive(Subcommand)]
pub enum Command {
    /// Serves the MCP tools of docs/mcp.md over stdio, on the local library.
    Mcp(McpArgs),
    /// Lists the library's animations, as a table or JSON.
    List(ListArgs),
    /// Writes an animation's files into a directory.
    Export(ExportArgs),
}

/// `life-pixel mcp [--library <dir>] [--allow-dir <dir>]…`.
#[derive(Args)]
pub struct McpArgs {
    /// The library folder; else `LIFE_PIXEL_LIBRARY`, else the default library.
    #[arg(long)]
    pub library: Option<PathBuf>,
    /// A directory `export` may also write to, beyond the working directory; repeatable.
    #[arg(long = "allow-dir")]
    pub allow_dir: Vec<PathBuf>,
}

/// `life-pixel list [--library <dir>] [--project <id>] [--query <text>] [--json]`.
#[derive(Args)]
pub struct ListArgs {
    /// The library folder; else `LIFE_PIXEL_LIBRARY`, else the default library.
    #[arg(long)]
    pub library: Option<PathBuf>,
    /// Only the animations of this project.
    #[arg(long)]
    pub project: Option<Uuid>,
    /// Only the animations whose title contains this text.
    #[arg(long)]
    pub query: Option<String>,
    /// Prints a JSON array instead of a table.
    #[arg(long)]
    pub json: bool,
}

/// `life-pixel export <id> --format … [--out] [--tag] [--scale] [--overwrite] [--library]`.
#[derive(Args)]
pub struct ExportArgs {
    /// The animation's id.
    pub id: Uuid,
    /// The format of the files.
    #[arg(long, value_enum)]
    pub format: ExportFormatArg,
    /// The library folder; else `LIFE_PIXEL_LIBRARY`, else the default library.
    #[arg(long)]
    pub library: Option<PathBuf>,
    /// Where to write the files; the working directory when omitted.
    #[arg(long = "out")]
    pub out: Option<PathBuf>,
    /// A tag whose frames to export; the whole animation when omitted.
    #[arg(long)]
    pub tag: Option<String>,
    /// How many times each pixel is repeated, for the image formats.
    #[arg(long)]
    pub scale: Option<u8>,
    /// Replaces an existing file instead of failing.
    #[arg(long)]
    pub overwrite: bool,
}

/// `export`'s `--format`: the names `ExportFormat` serializes.
#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum ExportFormatArg {
    /// The self-contained WebAssembly bundle and its loader.
    Wasm,
    /// An animated GIF.
    Gif,
    /// An animated PNG.
    Apng,
    /// Every frame in one PNG, with a JSON of where each one lies.
    SpriteSheet,
    /// A zip of one PNG per frame.
    PngFrames,
}

impl From<ExportFormatArg> for ExportFormat {
    fn from(format: ExportFormatArg) -> Self {
        match format {
            ExportFormatArg::Wasm => Self::Wasm,
            ExportFormatArg::Gif => Self::Gif,
            ExportFormatArg::Apng => Self::Apng,
            ExportFormatArg::SpriteSheet => Self::SpriteSheet,
            ExportFormatArg::PngFrames => Self::PngFrames,
        }
    }
}
