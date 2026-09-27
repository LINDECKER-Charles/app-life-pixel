//! The text grid: a cel as one string per row, top to bottom — the representation of MCP's
//! `write_frame` and of the sample animations.
//!
//! - **Single mode**, when the palette has at most 62 entries: one character per pixel — `.` for
//!   0, `1`–`9` for 1 to 9, `a`–`z` for 10 to 35, `A`–`Z` for 36 to 61.
//! - **Pair mode**, above: two lowercase hexadecimal digits per pixel, `00` for 0.

use crate::error::DocumentError;
use crate::model::{Cel, CelShape};

/// The characters of single mode, index 0 first.
const SINGLE_ALPHABET: &[u8; 62] =
    b".123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// The digits of pair mode, value 0 first.
const PAIR_DIGITS: &[u8; 16] = b"0123456789abcdef";
/// The radix of pair mode's digits.
const PAIR_RADIX: u8 = 16;

/// How many characters write one pixel.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Single,
    Pair,
}

impl Mode {
    fn chars_per_pixel(self) -> usize {
        match self {
            Self::Single => 1,
            Self::Pair => 2,
        }
    }

    fn decode(self, symbol: &[char]) -> Option<u8> {
        match (self, symbol) {
            (Self::Single, [character]) => single_index(*character),
            (Self::Pair, [high, low]) => Some(pair_digit(*high)? * PAIR_RADIX + pair_digit(*low)?),
            _ => None,
        }
    }

    fn write(self, index: u8, row: &mut String) {
        match self {
            Self::Single => row.push(char::from(SINGLE_ALPHABET[usize::from(index)])),
            Self::Pair => {
                row.push(char::from(PAIR_DIGITS[usize::from(index / PAIR_RADIX)]));
                row.push(char::from(PAIR_DIGITS[usize::from(index % PAIR_RADIX)]));
            }
        }
    }
}

/// The rows of `indices`, a cel or a composite of `shape`: single mode when the palette has at
/// most 62 entries, pair mode above.
#[must_use]
pub fn format(indices: &[u8], shape: CelShape) -> Vec<String> {
    let fits_single = shape.palette_len <= SINGLE_ALPHABET.len()
        && indices
            .iter()
            .all(|&index| usize::from(index) < SINGLE_ALPHABET.len());
    let mode = if fits_single {
        Mode::Single
    } else {
        Mode::Pair
    };
    let row_capacity = usize::from(shape.width) * mode.chars_per_pixel();
    indices
        .chunks(usize::from(shape.width).max(1))
        .map(|pixels| {
            let mut row = String::with_capacity(row_capacity);
            pixels.iter().for_each(|&index| mode.write(index, &mut row));
            row
        })
        .collect()
}

/// The cel of `rows`, for an animation of `shape`. The mode follows the length of the rows.
///
/// # Errors
///
/// [`DocumentError::GridSize`] when there are not `height` rows of `width` pixels in one mode,
/// [`DocumentError::GridCharacter`] for a character outside the mode's alphabet,
/// [`DocumentError::GridIndex`] for an index at or above the palette size.
pub fn parse<S: AsRef<str>>(rows: &[S], shape: CelShape) -> Result<Cel, DocumentError> {
    let rows: Vec<Vec<char>> = rows
        .iter()
        .map(|row| row.as_ref().chars().collect())
        .collect();
    let mode = mode_of(&rows, shape).ok_or(DocumentError::GridSize {
        width: shape.width,
        height: shape.height,
    })?;
    let reader = RowReader {
        mode,
        palette_len: shape.palette_len,
    };
    let mut indices = Vec::with_capacity(shape.pixel_count());
    for (row_number, row) in rows.iter().enumerate() {
        indices.extend(reader.read(row_number, row)?);
    }
    Ok(Cel::new(indices))
}

/// The mode every row is written in, when there are `height` of them.
fn mode_of(rows: &[Vec<char>], shape: CelShape) -> Option<Mode> {
    let width = usize::from(shape.width);
    let mode = match rows.first()?.len() {
        length if length == width => Mode::Single,
        length if length == width * Mode::Pair.chars_per_pixel() => Mode::Pair,
        _ => return None,
    };
    let row_length = width * mode.chars_per_pixel();
    let is_complete =
        rows.len() == usize::from(shape.height) && rows.iter().all(|row| row.len() == row_length);
    is_complete.then_some(mode)
}

/// Reads the rows of one grid, all in one mode.
struct RowReader {
    mode: Mode,
    palette_len: usize,
}

impl RowReader {
    fn read(&self, row: usize, characters: &[char]) -> Result<Vec<u8>, DocumentError> {
        let symbols = characters.chunks(self.mode.chars_per_pixel()).enumerate();
        symbols
            .map(|(column, symbol)| {
                let index = self
                    .mode
                    .decode(symbol)
                    .ok_or(DocumentError::GridCharacter { row, column })?;
                (usize::from(index) < self.palette_len)
                    .then_some(index)
                    .ok_or(DocumentError::GridIndex { row, column, index })
            })
            .collect()
    }
}

fn single_index(character: char) -> Option<u8> {
    let position = SINGLE_ALPHABET
        .iter()
        .position(|&symbol| char::from(symbol) == character)?;
    u8::try_from(position).ok()
}

fn pair_digit(character: char) -> Option<u8> {
    let position = PAIR_DIGITS
        .iter()
        .position(|&digit| char::from(digit) == character)?;
    u8::try_from(position).ok()
}
