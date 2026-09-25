//! The demo of `target/demo/`: the loader, each sample's WASM export, and `samples/demo.html`.

use std::path::Path;

use anyhow::{Context, Result};

/// Writes the demo into `<target_dir>/demo/`, replacing it if it already exists.
pub fn build(root: &Path, target_dir: &Path, exports: &[(&str, &[u8])]) -> Result<()> {
    let demo_dir = target_dir.join("demo");
    std::fs::create_dir_all(&demo_dir)
        .with_context(|| format!("creating {}", demo_dir.display()))?;
    copy(&root.join("player-js/life-pixel.js"), &demo_dir.join("life-pixel.js"))?;
    copy(&root.join("samples/demo.html"), &demo_dir.join("demo.html"))?;
    for (name, bytes) in exports {
        let path = demo_dir.join(format!("{name}.wasm"));
        std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn copy(from: &Path, to: &Path) -> Result<()> {
    std::fs::copy(from, to)
        .with_context(|| format!("copying {} to {}", from.display(), to.display()))?;
    Ok(())
}
