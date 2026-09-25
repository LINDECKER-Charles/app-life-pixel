//! The editing operations: every operation and its undo, restoring the document byte for byte;
//! the frame and tag rules; each error, leaving the document unchanged; previews; the history;
//! the import of PNG images and sprite sheets; golden images of scripted sequences.

// `cfg(test)` holds for every test target; it lets the helpers unwrap as the tests do.
#[cfg(test)]
mod frames;
#[cfg(test)]
mod golden;
#[cfg(test)]
mod history;
#[cfg(test)]
mod import;
#[cfg(test)]
mod layers;
#[cfg(test)]
mod palette;
#[cfg(test)]
mod pixels;
#[cfg(test)]
mod shape;
#[cfg(test)]
mod support;
