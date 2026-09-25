use std::{fs, path::Path};

use anyhow::Context;
use cargo_metadata::{DependencyKind, MetadataCommand, Package};

use super::Member;

/// Where the loader's manifest lives, relative to the workspace root.
const PLAYER_JS_MANIFEST: &str = "player-js/package.json";

/// The packages the boundary covers: the Cargo workspace's members and, once it exists, the
/// npm package of the loader.
#[derive(Clone, Debug)]
pub struct Workspace {
    /// The members of the Cargo workspace.
    pub members: Vec<Member>,
    /// `player-js`, when its manifest exists.
    pub player_js: Option<Member>,
}

impl Workspace {
    /// Reads the workspace through `cargo metadata`, then `player-js/package.json`.
    pub fn load() -> anyhow::Result<Self> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("running cargo metadata")?;
        let members = metadata
            .workspace_packages()
            .into_iter()
            .map(member_of)
            .collect();
        let player_js = load_player_js(metadata.workspace_root.as_std_path())?;
        Ok(Self { members, player_js })
    }
}

fn member_of(package: &Package) -> Member {
    let dependencies = package
        .dependencies
        .iter()
        .filter(|dependency| dependency.kind != DependencyKind::Development)
        .map(|dependency| dependency.name.clone())
        .collect();
    Member {
        name: package.name.to_string(),
        license: package.license.clone(),
        dependencies,
    }
}

fn load_player_js(workspace_root: &Path) -> anyhow::Result<Option<Member>> {
    let path = workspace_root.join(PLAYER_JS_MANIFEST);
    if !path.exists() {
        return Ok(None);
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("reading {PLAYER_JS_MANIFEST}"))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&text).with_context(|| format!("parsing {PLAYER_JS_MANIFEST}"))?;
    let license = manifest
        .get("license")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    Ok(Some(Member {
        name: PLAYER_JS_MANIFEST.to_owned(),
        license,
        dependencies: Vec::new(),
    }))
}
