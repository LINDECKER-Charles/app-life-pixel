//! The MCP endpoint's metrics of the catalogue of docs/admin-console.md: the tool calls by tool
//! and outcome, and the exports the signed links download, by format and outcome.

use std::time::Duration;

use life_pixel_mcp::ExportFormat;
use metrics::{Unit, counter, describe_counter, describe_histogram, histogram};

/// Tool calls, by `tool` and `outcome`.
pub const MCP_TOOL_CALLS_TOTAL: &str = "mcp_tool_calls_total";
/// Time to answer a tool call, by `tool`.
pub const MCP_TOOL_DURATION_SECONDS: &str = "mcp_tool_duration_seconds";
/// Exports downloaded through a signed link, by `format` and `outcome`.
pub const EXPORTS_TOTAL: &str = "exports_total";
/// Time to compile an export again for its download, by `format`.
pub const EXPORT_DURATION_SECONDS: &str = "export_duration_seconds";
/// Bytes of a downloaded export file, by `format`.
pub const EXPORT_SIZE_BYTES: &str = "export_size_bytes";

/// How a call or an export ended: the `outcome` label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// It succeeded.
    Ok,
    /// It failed, whatever the code.
    Error,
}

impl Outcome {
    /// The value of the `outcome` label, and of the `mcp_tool_called` event's property.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

/// Describes the MCP metrics to the recorder, once it is installed.
pub fn describe() {
    describe_counter!(MCP_TOOL_CALLS_TOTAL, "MCP tool calls, by tool and outcome");
    describe_histogram!(
        MCP_TOOL_DURATION_SECONDS,
        Unit::Seconds,
        "Time to answer an MCP tool call"
    );
    describe_counter!(EXPORTS_TOTAL, "Exports downloaded, by format and outcome");
    describe_histogram!(
        EXPORT_DURATION_SECONDS,
        Unit::Seconds,
        "Time to compile an export for its download"
    );
    describe_histogram!(
        EXPORT_SIZE_BYTES,
        Unit::Bytes,
        "Bytes of a downloaded export file"
    );
}

/// Records one call of `tool`, ended with `outcome` after `elapsed`.
pub fn record_call(tool: &str, outcome: Outcome, elapsed: Duration) {
    let tool = tool.to_owned();
    counter!(MCP_TOOL_CALLS_TOTAL, "tool" => tool.clone(), "outcome" => outcome.label())
        .increment(1);
    histogram!(MCP_TOOL_DURATION_SECONDS, "tool" => tool).record(elapsed.as_secs_f64());
}

/// Records one download of the format `format`, `Some` of its file's bytes when it succeeded.
pub fn record_export(format: ExportFormat, (bytes, elapsed): (Option<usize>, Duration)) {
    let format = format_label(format);
    let outcome = if bytes.is_some() {
        Outcome::Ok
    } else {
        Outcome::Error
    };
    counter!(EXPORTS_TOTAL, "format" => format, "outcome" => outcome.label()).increment(1);
    let Some(bytes) = bytes else {
        return;
    };
    histogram!(EXPORT_DURATION_SECONDS, "format" => format).record(elapsed.as_secs_f64());
    #[allow(clippy::cast_precision_loss)] // A histogram is a float; files stay far below 2^53.
    histogram!(EXPORT_SIZE_BYTES, "format" => format).record(bytes as f64);
}

/// The `format` label: the format's name in the API.
#[must_use]
pub const fn format_label(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Wasm => "wasm",
        ExportFormat::Gif => "gif",
        ExportFormat::Apng => "apng",
        ExportFormat::SpriteSheet => "sprite_sheet",
        ExportFormat::PngFrames => "png_frames",
    }
}
