//! Binaire client desktop Teleportal Remote (Tauri v2).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    teleportal_desktop_client_lib::run();
}
