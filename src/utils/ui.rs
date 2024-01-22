use eframe::egui::{self};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs::{self};
use std::time::SystemTime;

use crate::utils::cut::CutHandler;

use super::plot_manager::PlotManager;

use crate::histograms::histogram_creation::add_histograms;
use crate::utils::histogrammer::Histogrammer;

pub struct MyApp {
    selected_directory: Option<PathBuf>,
    file_paths: Vec<PathBuf>,
    histograms_loaded: bool,
    plot_manager: PlotManager,
    cut_file_path: Option<PathBuf>,
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            selected_directory: None, 
            file_paths: Vec::new(),
            histograms_loaded: false,
            plot_manager: PlotManager::new(Histogrammer::new(), CutHandler::new()),
            cut_file_path: None,
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

                if ui.button("Select Cut File").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.cut_file_path = Some(path);
                    }
                }
    
                // Display the selected cut file name
                if let Some(cut_path) = &self.cut_file_path {
                    if let Some(file_name) = cut_path.file_name().and_then(|name| name.to_str()) {
                        ui.label(format!("Selected Cut File: {}", file_name));
                    } else {
                        ui.label("Selected Cut File: Invalid file name");
                    }
    
                    // Add a button to remove the selected cut file
                    if ui.button("Remove Cut File").clicked() {
                        self.cut_file_path = None;
                    }
                }
                
                ui.separator();

                if ui.button("Load Histograms").clicked() {
                    self.histograms_loaded = false;

                    if !self.file_paths.is_empty() {
                        // Convert Vec<PathBuf> to Arc<[PathBuf]>
                        let paths_arc: Arc<[PathBuf]> = Arc::from(self.file_paths.clone().into_iter().collect::<Box<[_]>>());

                        match add_histograms(paths_arc.clone(), self.cut_file_path.clone()) {
                            Ok(histogrammer) => {
                                // self.histogrammer = histogrammer;
                                self.plot_manager.histogrammer = histogrammer;
                                self.histograms_loaded = true;
                            }
                            Err(e) => {
                                eprintln!("Failed to load histograms: {:?}", e);
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

        if self.histograms_loaded {

            egui::SidePanel::right("histograms").show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.plot_manager.render_buttons(ui);
                });
            });

            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                
                self.plot_manager.cutter.cut_handler_ui(ui);
    
            });

            // add centralpanel last
            egui::CentralPanel::default().show(ctx, |ui| {
                self.plot_manager.render_selected_histograms(ui);
            });

        }
    }
}