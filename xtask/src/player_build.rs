// The build of the player, shared by `cargo xtask build-player` and the compiler's build script,
// which both pull this file in with `include!`. It holds one function, uses `std` alone through
// full paths, and imports nothing, so that it compiles wherever it is included.

/// Builds `life-pixel-player` for `wasm32-unknown-unknown` with the `player` profile into
/// `target_dir`, and returns the path of the module.
///
/// The bytes depend on the sources and the toolchain alone: `--remap-path-prefix` replaces the
/// workspace and `CARGO_HOME` with fixed paths, `CARGO_ENCODED_RUSTFLAGS` overrides every other
/// source of compiler flags, and the variables through which an outer build would change the
/// compiler, the target or the target directory are removed from the command's environment.
fn build_player(
    workspace_root: &std::path::Path,
    target_dir: &std::path::Path,
) -> std::io::Result<std::path::PathBuf> {
    let cargo_home = std::env::var_os("CARGO_HOME").map(std::path::PathBuf::from);
    let cargo_home = cargo_home.or_else(|| Some(std::env::home_dir()?.join(".cargo")));
    let mut rustflags = std::ffi::OsString::from("--remap-path-prefix=");
    rustflags.push(workspace_root);
    rustflags.push("=/life-pixel");
    if let Some(cargo_home) = cargo_home {
        rustflags.push("\u{1f}--remap-path-prefix=");
        rustflags.push(cargo_home);
        rustflags.push("=/cargo-home");
    }
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = std::process::Command::new(cargo);
    command.current_dir(workspace_root).env("CARGO_ENCODED_RUSTFLAGS", rustflags);
    command.args(["build", "--package", "life-pixel-player", "--lib", "--locked"]);
    command.args(["--target", "wasm32-unknown-unknown", "--profile", "player", "--target-dir"]);
    command.arg(target_dir);
    let compiler = ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER", "RUSTFLAGS", "CARGO_INCREMENTAL"];
    for variable in compiler.into_iter().chain(["CARGO_BUILD_TARGET", "CARGO_TARGET_DIR"]) {
        command.env_remove(variable);
    }
    match command.status()? {
        status if status.success() => {
            Ok(target_dir.join("wasm32-unknown-unknown/player/life_pixel_player.wasm"))
        }
        status => Err(std::io::Error::other(format!("building life-pixel-player: {status}"))),
    }
}
