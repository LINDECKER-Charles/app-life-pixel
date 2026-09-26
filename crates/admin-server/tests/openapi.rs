//! The committed description is the one the code writes: regenerate it with
//! `cargo run -p life-pixel-admin-server -- openapi > crates/admin-server/openapi.json`.

#![allow(clippy::unwrap_used)]

use life_pixel_admin_server::openapi;

#[test]
fn the_committed_description_is_current() {
    let committed = include_str!("../openapi.json");
    assert!(
        openapi::to_json().unwrap() == committed,
        "crates/admin-server/openapi.json is stale: regenerate it"
    );
}
