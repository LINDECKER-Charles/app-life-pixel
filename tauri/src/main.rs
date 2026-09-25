//! The desktop app's binary: everything is in the library, which a mobile build loads too.

// No console window beside the app on Windows, except in debug builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    life_pixel_desktop::run();
}
