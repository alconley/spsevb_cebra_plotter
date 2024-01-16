use eframe::egui::{self};
use super::histogrammer::{Histogrammer};

use crate::histograms::sps::add_sps_histograms;
use crate::histograms::cebra::{Cebr3Detector, cebra_detector_ui, add_cebra_histograms};
use crate::histograms::sps_cebra::{Cebr3DetectorWithSPS, sps_cebra_detector_ui, add_sps_cebra_histograms};

use std::sync::Arc;
use std::path::PathBuf;
use std::fs::{self};
use std::time::SystemTime;

#[derive(Default)]
pub struct MyApp {
    selected_directory: Option<PathBuf>,
    file_paths: Vec<PathBuf>,
    selected_option: String,
    cebr3_detectors: Vec<Cebr3Detector>,  
    sps_cebr3_detectors: Vec<Cebr3DetectorWithSPS>,  
    histograms_loaded: bool,
    histogrammer: Histogrammer,
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            selected_directory: None, 
            file_paths: Vec::new(),
            selected_option: "SPS".to_string(), // Default to "SPS" histograms
            cebr3_detectors: Vec::new(),
            sps_cebr3_detectors: Vec::new(),
            histograms_loaded: false,
            histogrammer: Histogrammer::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::SidePanel::left("files").show(ctx, |ui| {

            if ui.button("Open Directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.selected_directory = Some(path);
                }
            }

            ui.separator();

            // Function to get the modification time of a file
            fn get_modification_time(path: &PathBuf) -> Option<SystemTime> {
                fs::metadata(path).ok().and_then(|metadata| metadata.modified().ok())
            }

            if let Some(dir) = &self.selected_directory {

                if ui.button("Load Histograms").clicked() {
                    self.histograms_loaded = false;

                    if !self.file_paths.is_empty() {
                        // Convert Vec<PathBuf> to Arc<[PathBuf]>
                        let paths_arc: Arc<[PathBuf]> = Arc::from(self.file_paths.clone().into_iter().collect::<Box<[_]>>());

                        if self.selected_option == "SPS" {
                            match add_sps_histograms(paths_arc.clone()) {
                                Ok(histogrammer) => {
                                    self.histogrammer = histogrammer;
                                    self.histograms_loaded = true;
                                }
                                Err(e) => {
                                    eprintln!("Failed to load histograms: {:?}", e);
                                }
                            }
                        }

                        if self.selected_option == "CeBrA" {
                            match add_cebra_histograms(paths_arc.clone(), &mut self.cebr3_detectors) {
                                Ok(histogrammer) => {
                                    self.histogrammer = histogrammer;
                                    self.histograms_loaded = true;
                                }
                                Err(e) => {
                                    eprintln!("Failed to load histograms: {:?}", e);
                                }
                            }
                        }

                        if self.selected_option == "SPS+CeBrA" {
                            match add_sps_cebra_histograms(paths_arc.clone(), &mut self.sps_cebr3_detectors) {
                                Ok(histogrammer) => {
                                    self.histogrammer = histogrammer;
                                    self.histograms_loaded = true;
                                }
                                Err(e) => {
                                    eprintln!("Failed to load histograms: {:?}", e);
                                }
                            }
                        }

                    }
                }

                ui.separator();

                ui.label("Files in directory");
                if ui.button("Select All").clicked() {
                    if let Ok(entries) = fs::read_dir(dir) {
                        for entry in entries.filter_map(Result::ok) {
                            let path = entry.path();
                            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                                if !self.file_paths.contains(&path) {
                                    self.file_paths.push(path);
                                }
                            }
                        }
                    }
                }
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    
                    // Attempt to read the directory
                    match fs::read_dir(dir) {
                        Ok(entries) => {
                            let mut files: Vec<_> = entries
                                .filter_map(Result::ok)
                                .filter(|entry| {
                                    entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("parquet")
                                })
                                .collect();
            
                            // Sort files by modification time
                            files.sort_by(|a, b| {
                                let a_time = get_modification_time(&a.path()).unwrap_or(SystemTime::UNIX_EPOCH);
                                let b_time = get_modification_time(&b.path()).unwrap_or(SystemTime::UNIX_EPOCH);
                                b_time.cmp(&a_time) // Sorting in reverse order
                            });
            
                            // Display the files
                            for entry in files {
                                let path = entry.path();
                                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                                    let file_name_display = file_name.strip_suffix(".parquet").unwrap_or(file_name);
                                    if ui.selectable_label(self.file_paths.contains(&path), file_name_display).clicked() {
                                        if self.file_paths.contains(&path) {
                                            self.file_paths.retain(|p| p != &path);
                                        } else {
                                            self.file_paths.push(path);
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Handle the error case here
                            ui.label("Failed to read directory");
                        }
                    }
                });
            
            }

        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {

            ui.horizontal(|ui| {
                // SPS SelectableLabel
                if ui.selectable_label(self.selected_option == "SPS", "SPS").clicked() {
                    self.selected_option = "SPS".to_string();
                }

                // CeBrA SelectableLabel
                if ui.selectable_label(self.selected_option == "CeBrA", "CeBrA").clicked() {
                    self.selected_option = "CeBrA".to_string();
                }

                // SPS+CeBrA SelectableLabel
                if ui.selectable_label(self.selected_option == "SPS+CeBrA", "SPS+CeBrA").clicked() {
                    self.selected_option = "SPS+CeBrA".to_string();
                }

            });

            // Call cebra_detector_ui outside the selectable label's scope
            if self.selected_option == "CeBrA" {
                cebra_detector_ui(&mut self.cebr3_detectors, ui);
            }

            // Call cebra_detector_ui outside the selectable label's scope
            if self.selected_option == "SPS+CeBrA" {
                sps_cebra_detector_ui(&mut self.sps_cebr3_detectors, ui);
            }
        
         });

        if self.histograms_loaded {

            egui::SidePanel::right("histograms").show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.histogrammer.render_buttons(ui);
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                // self.histogrammer.render_selected_histogram(ui);
                self.histogrammer.render_selected_histograms(ui);
            });
        }
    }

}