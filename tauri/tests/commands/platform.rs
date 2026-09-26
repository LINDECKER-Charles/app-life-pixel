//! The platform's description.

use life_pixel_desktop::commands::platform::platform_info;

use crate::harness::{FakeDialogs, Harness};

#[tokio::test]
async fn the_platform_names_its_version_system_and_library() {
    let harness = Harness::new();
    let info = serde_json::to_value(platform_info(harness.state()).await.unwrap()).unwrap();
    assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(info["os"], std::env::consts::OS);
    assert_eq!(
        info["libraryPath"],
        harness.default_library().display().to_string()
    );
    assert_eq!(info["cliPath"], serde_json::Value::Null);
}

#[tokio::test]
async fn the_platform_names_the_bundled_cli() {
    let cli = std::env::temp_dir().join("bin").join("life-pixel");
    let path = cli.clone();
    let harness = Harness::with(FakeDialogs::default(), |options| {
        options.cli_path = Some(path)
    });
    let info = serde_json::to_value(platform_info(harness.state()).await.unwrap()).unwrap();
    assert_eq!(info["cliPath"], cli.display().to_string());
}
