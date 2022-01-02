#![windows_subsystem = "windows"]
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![warn(clippy::all, rust_2018_idioms)]
use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = monsterbook::app::App::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(320.0, 240.0)),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
