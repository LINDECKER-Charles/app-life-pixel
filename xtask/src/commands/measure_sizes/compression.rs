//! Raw, gzip and brotli sizes of a blob — the three columns of docs/export.md's size table.

use std::io::Write as _;

use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::GzEncoder;

/// The gzip level docs/export.md measures at: "gzipped at level 9".
const GZIP_LEVEL: u32 = 9;
/// The brotli quality docs/export.md measures at: "with brotli at level 11".
const BROTLI_QUALITY: u32 = 11;
/// Brotli's window size, the largest without the "large window" extension.
const BROTLI_LG_WINDOW: u32 = 24;
/// The brotli writer's internal buffer; it does not bound the output.
const BROTLI_BUFFER_BYTES: usize = 4096;

/// The three sizes of one artefact, in bytes.
#[derive(Clone, Copy, Debug)]
pub struct Sizes {
    /// The artefact's own size.
    pub raw: usize,
    /// Gzipped at [`GZIP_LEVEL`].
    pub gzip: usize,
    /// Compressed with brotli at [`BROTLI_QUALITY`].
    pub brotli: usize,
}

impl Sizes {
    /// Measures `bytes` the three ways docs/export.md's table reports. Deterministic: the same
    /// bytes always compress to the same sizes.
    pub fn measure(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            raw: bytes.len(),
            gzip: gzip_size(bytes).context("gzipping")?,
            brotli: brotli_size(bytes).context("brotli-compressing")?,
        })
    }
}

fn gzip_size(bytes: &[u8]) -> Result<usize> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(GZIP_LEVEL));
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?.len())
}

fn brotli_size(bytes: &[u8]) -> Result<usize> {
    let mut writer = brotli::CompressorWriter::new(
        Vec::new(),
        BROTLI_BUFFER_BYTES,
        BROTLI_QUALITY,
        BROTLI_LG_WINDOW,
    );
    writer.write_all(bytes)?;
    Ok(writer.into_inner().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_never_grows_past_a_few_bytes_of_overhead() {
        let bytes = vec![b'a'; 4096];
        let sizes = Sizes::measure(&bytes).unwrap();
        assert_eq!(sizes.raw, 4096);
        assert!(sizes.gzip < sizes.raw);
        assert!(sizes.brotli < sizes.gzip);
    }

    #[test]
    fn measuring_the_same_bytes_twice_gives_the_same_sizes() {
        let bytes: Vec<u8> = (0..2048).map(|value| (value % 251) as u8).collect();
        let first = Sizes::measure(&bytes).unwrap();
        let second = Sizes::measure(&bytes).unwrap();
        assert_eq!(first.raw, second.raw);
        assert_eq!(first.gzip, second.gzip);
        assert_eq!(first.brotli, second.brotli);
    }
}
