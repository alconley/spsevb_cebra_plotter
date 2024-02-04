#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod utils;
mod histograms;

use crate::utils::ui::MyApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1250.0, 750.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Histogram Viewer",
        options,
        // Box::new(|_cc| Box::<MyApp>::default()),
        Box::new(|_cc| Box::new(<MyApp>::new())),


    )
}

