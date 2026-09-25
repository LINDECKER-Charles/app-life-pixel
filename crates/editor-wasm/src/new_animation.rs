//! What `create` receives: the interface's `NewAnimationOptions`, checked into `core`'s
//! [`NewAnimation`].

use life_pixel_core::{DocumentError, Name, NewAnimation, Palette, Rgba};
use serde::Deserialize;

/// The interface's `NewAnimationOptions`. Numbers are read wider than the model holds them, so that
/// an out-of-range value fails with its own code rather than as a malformed request.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewAnimationOptions {
    /// The title.
    pub title: String,
    /// The canvas width, in pixels.
    pub width: u32,
    /// The canvas height, in pixels.
    pub height: u32,
    /// The name of the first layer, translated by the interface.
    pub layer_name: String,
    /// The duration of the first frame; `core`'s default when absent.
    pub frame_duration_ms: Option<u32>,
    /// The palette, colours written `#rrggbbaa`; `core`'s default when absent.
    pub palette: Option<Vec<String>>,
}

impl TryFrom<NewAnimationOptions> for NewAnimation {
    type Error = DocumentError;

    fn try_from(options: NewAnimationOptions) -> Result<Self, Self::Error> {
        let side = |value: u32| u16::try_from(value).map_err(|_| DocumentError::CanvasSize);
        let duration = options.frame_duration_ms.map(u16::try_from).transpose();
        Ok(Self {
            title: Name::new(&options.title)?,
            width: side(options.width)?,
            height: side(options.height)?,
            layer_name: Name::new(&options.layer_name)?,
            frame_duration_ms: duration.map_err(|_| DocumentError::FrameDuration)?,
            palette: options.palette.as_deref().map(read_palette).transpose()?,
        })
    }
}

fn read_palette(colours: &[String]) -> Result<Palette, DocumentError> {
    let entries = colours.iter().map(|colour| colour.parse::<Rgba>());
    let entries = entries.collect::<Result<Vec<_>, _>>();
    Palette::new(entries.map_err(|_| DocumentError::Palette)?)
}
