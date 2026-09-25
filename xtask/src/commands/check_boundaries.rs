use anyhow::bail;

use crate::boundaries::{Workspace, check};

/// Checks the licence boundary between the MIT and the AGPL code.
///
/// `life-pixel-format` has no dependency, `life-pixel-player` depends on `life-pixel-format`
/// only, both declare MIT, every other workspace member declares AGPL-3.0-only, and
/// `player-js/package.json`, once it exists, declares MIT.
#[derive(clap::Args)]
pub struct CheckBoundaries {}

impl CheckBoundaries {
    /// Reads the workspace, then fails on the first report of a broken boundary.
    pub fn run(self) -> anyhow::Result<()> {
        let violations = check(&Workspace::load()?);
        if violations.is_empty() {
            println!("check-boundaries: every boundary holds");
            return Ok(());
        }
        for violation in &violations {
            eprintln!("check-boundaries: {violation}");
        }
        bail!("{} boundary violation(s)", violations.len())
    }
}
