//! The classic formats: files that image viewers, browsers and game engines already read. Each
//! export renders the frames of its [range](ClassicOptions::tag) through `life-pixel-core`,
//! scales them, and encodes them; none adds a pixel rule of its own.

mod apng;
mod gif;
mod indexed_png;
mod options;
mod plan;
mod png_frames;
mod range;
mod scale;
mod sprite_sheet;

pub use apng::export_apng;
pub use gif::export_gif;
pub use options::ClassicOptions;
pub use png_frames::export_png_frames;
pub use sprite_sheet::export_sprite_sheet;
