//! Every product limit, written once. Validation, MCP schemas and the interface read them from
//! here — the interface through [`Limits::current`] — and never repeat one as a literal.

use serde::Serialize;

/// The smallest side of a canvas, in pixels.
pub const CANVAS_MIN_SIDE: u16 = 1;
/// The largest side of a canvas, in pixels.
pub const CANVAS_MAX_SIDE: u16 = 512;
/// The most frames an animation holds.
pub const MAX_FRAMES: usize = 1_024;
/// The most layers an animation holds.
pub const MAX_LAYERS: usize = 64;
/// The most tags an animation holds.
pub const MAX_TAGS: usize = 64;
/// The most pixels of all non-empty cels together.
pub const MAX_CEL_PIXELS: usize = 16_777_216;
/// The most palette entries, entry 0 included.
pub const MAX_PALETTE_ENTRIES: usize = 256;
/// The shortest frame duration, in milliseconds.
pub const MIN_FRAME_DURATION_MS: u16 = 10;
/// The longest frame duration, in milliseconds.
pub const MAX_FRAME_DURATION_MS: u16 = 65_535;
/// The duration of a new frame, in milliseconds.
pub const DEFAULT_FRAME_DURATION_MS: u16 = 100;
/// The most characters of a title, project or layer name, after trimming.
pub const NAME_MAX_CHARS: usize = 100;
/// The most characters of a tag name.
pub const TAG_NAME_MAX_CHARS: usize = 32;
/// The most bytes of a serialized document.
pub const MAX_DOCUMENT_BYTES: usize = 33_554_432;
/// The largest side of an imported image, in pixels.
pub const IMPORT_MAX_SIDE: u32 = 4_096;
/// The most bytes of an imported image file.
pub const IMPORT_MAX_BYTES: usize = 16_777_216;
/// The most points of one stroke.
pub const STROKE_MAX_POINTS: usize = 10_000;
/// The most operations of one MCP `draw` call.
pub const DRAW_MAX_OPERATIONS: usize = 1_000;
/// The most undo steps kept.
pub const HISTORY_MAX_STEPS: usize = 200;
/// The most bytes of inverses kept for undo.
pub const HISTORY_MAX_BYTES: usize = 67_108_864;
/// The smallest export scale.
pub const EXPORT_MIN_SCALE: u32 = 1;
/// The largest export scale.
pub const EXPORT_MAX_SCALE: u32 = 16;
/// The largest side of an exported image, in pixels, after scaling.
pub const EXPORT_MAX_SIDE: u32 = 8_192;
/// The largest side of a preview image, in pixels.
pub const PREVIEW_MAX_SIDE: u32 = 1_024;
/// The most bytes of a preview image.
pub const PREVIEW_MAX_BYTES: usize = 1_048_576;
/// The fewest characters of a password.
pub const PASSWORD_MIN_CHARS: usize = 12;
/// The most characters of a password.
pub const PASSWORD_MAX_CHARS: usize = 128;
/// The most characters of an email address.
pub const EMAIL_MAX_CHARS: usize = 254;
/// The most characters of a support message.
pub const SUPPORT_MESSAGE_MAX_CHARS: usize = 5_000;
/// The most bytes of a support screenshot.
pub const SCREENSHOT_MAX_BYTES: usize = 5_242_880;
/// The largest side of a support screenshot, in pixels.
pub const SCREENSHOT_MAX_SIDE: u32 = 4_096;
/// The most characters of an access token's name.
pub const TOKEN_NAME_MAX_CHARS: usize = 60;
/// The lifetimes an access token may be given, in days.
pub const TOKEN_EXPIRY_DAYS: [u16; 3] = [30, 90, 365];
/// The most active access tokens of one account.
pub const MAX_ACTIVE_TOKENS: usize = 20;
/// The page size of an HTTP collection when none is asked for.
pub const PAGE_SIZE_DEFAULT: usize = 50;
/// The largest page size of an HTTP collection.
pub const PAGE_SIZE_MAX: usize = 100;
/// The page size of an MCP listing when none is asked for.
pub const MCP_PAGE_SIZE_DEFAULT: usize = 20;
/// The largest page size of an MCP listing.
pub const MCP_PAGE_SIZE_MAX: usize = 50;

/// Every limit above, one field per constant, serialized in camelCase: what the engine gives the
/// interface, so that no front-end code repeats a limit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)] // Each field is the constant of the same name, documented above.
pub struct Limits {
    pub canvas_min_side: u16,
    pub canvas_max_side: u16,
    pub max_frames: usize,
    pub max_layers: usize,
    pub max_tags: usize,
    pub max_cel_pixels: usize,
    pub max_palette_entries: usize,
    pub min_frame_duration_ms: u16,
    pub max_frame_duration_ms: u16,
    pub default_frame_duration_ms: u16,
    pub name_max_chars: usize,
    pub tag_name_max_chars: usize,
    pub max_document_bytes: usize,
    pub import_max_side: u32,
    pub import_max_bytes: usize,
    pub stroke_max_points: usize,
    pub draw_max_operations: usize,
    pub history_max_steps: usize,
    pub history_max_bytes: usize,
    pub export_min_scale: u32,
    pub export_max_scale: u32,
    pub export_max_side: u32,
    pub preview_max_side: u32,
    pub preview_max_bytes: usize,
    pub password_min_chars: usize,
    pub password_max_chars: usize,
    pub email_max_chars: usize,
    pub support_message_max_chars: usize,
    pub screenshot_max_bytes: usize,
    pub screenshot_max_side: u32,
    pub token_name_max_chars: usize,
    pub token_expiry_days: [u16; 3],
    pub max_active_tokens: usize,
    pub page_size_default: usize,
    pub page_size_max: usize,
    pub mcp_page_size_default: usize,
    pub mcp_page_size_max: usize,
}

impl Limits {
    /// The limits of this build.
    #[must_use]
    #[allow(clippy::too_many_lines)] // One line per constant: splitting it would hide one.
    pub const fn current() -> Self {
        Self {
            canvas_min_side: CANVAS_MIN_SIDE,
            canvas_max_side: CANVAS_MAX_SIDE,
            max_frames: MAX_FRAMES,
            max_layers: MAX_LAYERS,
            max_tags: MAX_TAGS,
            max_cel_pixels: MAX_CEL_PIXELS,
            max_palette_entries: MAX_PALETTE_ENTRIES,
            min_frame_duration_ms: MIN_FRAME_DURATION_MS,
            max_frame_duration_ms: MAX_FRAME_DURATION_MS,
            default_frame_duration_ms: DEFAULT_FRAME_DURATION_MS,
            name_max_chars: NAME_MAX_CHARS,
            tag_name_max_chars: TAG_NAME_MAX_CHARS,
            max_document_bytes: MAX_DOCUMENT_BYTES,
            import_max_side: IMPORT_MAX_SIDE,
            import_max_bytes: IMPORT_MAX_BYTES,
            stroke_max_points: STROKE_MAX_POINTS,
            draw_max_operations: DRAW_MAX_OPERATIONS,
            history_max_steps: HISTORY_MAX_STEPS,
            history_max_bytes: HISTORY_MAX_BYTES,
            export_min_scale: EXPORT_MIN_SCALE,
            export_max_scale: EXPORT_MAX_SCALE,
            export_max_side: EXPORT_MAX_SIDE,
            preview_max_side: PREVIEW_MAX_SIDE,
            preview_max_bytes: PREVIEW_MAX_BYTES,
            password_min_chars: PASSWORD_MIN_CHARS,
            password_max_chars: PASSWORD_MAX_CHARS,
            email_max_chars: EMAIL_MAX_CHARS,
            support_message_max_chars: SUPPORT_MESSAGE_MAX_CHARS,
            screenshot_max_bytes: SCREENSHOT_MAX_BYTES,
            screenshot_max_side: SCREENSHOT_MAX_SIDE,
            token_name_max_chars: TOKEN_NAME_MAX_CHARS,
            token_expiry_days: TOKEN_EXPIRY_DAYS,
            max_active_tokens: MAX_ACTIVE_TOKENS,
            page_size_default: PAGE_SIZE_DEFAULT,
            page_size_max: PAGE_SIZE_MAX,
            mcp_page_size_default: MCP_PAGE_SIZE_DEFAULT,
            mcp_page_size_max: MCP_PAGE_SIZE_MAX,
        }
    }
}
