// Standard library imports
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

// External crates imports
use egui::Ui;
use polars::prelude::*;
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use serde_yaml;

// Local crate/module imports
use crate::utils::histogrammer::{Histogrammer};

#[derive(Serialize, Deserialize)]
pub struct Cebr3Detector {
    number: i32,
    gain_matched_values: [f64; 2],  // Tuple for 'm' and 'b'
    energy_calibration_values: [f64; 3],  // Tuple for 'a', 'b', and 'c'
}

pub fn cebra_detector_ui(detectors: &mut Vec<Cebr3Detector>, ui: &mut Ui) {
    // Loop through each detector
    for detector in detectors.iter_mut() {
        // Implement the UI for each detector
        ui.horizontal(|ui| {
            ui.label("Detector Number:");
            ui.add(egui::DragValue::new(&mut detector.number)
                .speed(0.1)
                .clamp_range(0..=6)  // Range from 0 to 6, since at the moment there are only 7 detectors
                ); 

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Gain Matched Values: y=");
                ui.add(egui::DragValue::new(&mut detector.gain_matched_values[0]).max_decimals(10).speed(0.1));
                ui.label("x+");
                ui.add(egui::DragValue::new(&mut detector.gain_matched_values[1]).max_decimals(10).speed(0.1));  
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Energy Calibration Values: y=");
                ui.add(egui::DragValue::new(&mut detector.energy_calibration_values[0]).max_decimals(10).speed(0.1));
                ui.label("x²+");
                ui.add(egui::DragValue::new(&mut detector.energy_calibration_values[1]).max_decimals(10).speed(0.1));  
                ui.label("x+");
                ui.add(egui::DragValue::new(&mut detector.energy_calibration_values[2]).max_decimals(10).speed(0.1)); 
            });

        });

    }

    ui.horizontal(|ui| {

        let max_det_number = detectors.iter().map(|d| d.number).max().unwrap_or(-1);

        // Check if the maximum number is less than 6 before showing the button
        if max_det_number < 6 {
            if ui.button("Add Detector").clicked() {
                detectors.push(Cebr3Detector {
                    number: max_det_number + 1, // Increment the maximum number
                    gain_matched_values: [1.0, 0.0],
                    energy_calibration_values: [0.0, 1.0, 0.0],
                });
            }
        } else {
            // Optional: show a disabled button or a label explaining why adding more is not possible
            ui.add_enabled(false, egui::Button::new("Add Detector"));
        }

        ui.separator();

        if ui.button("Save Calibration Settings").clicked() {
            if let Err(e) = save_cebra_settings_with_dialog(detectors) {
                eprintln!("Failed to save detectors: {}", e);
            }
        }

        ui.separator();

        if ui.button("Load Calibration Settings").clicked() {
            match load_cebra_settings_with_dialog() {
                Ok(loaded_detectors) => {
                    *detectors = loaded_detectors;
                },
                Err(e) => {
                    eprintln!("Failed to load detectors: {}", e);
                }
            }
        }

    });

}

fn save_cebra_settings_with_dialog(detectors: &[Cebr3Detector]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(file_path) = FileDialog::new()
        .set_file_name("CeBrA_Calibration.yaml")  // Suggest a default file name
        .add_filter("YAML Files", &["yaml", "yml"])  // Add a filter for YAML files
        .save_file() {

        let serialized = serde_yaml::to_string(detectors)?;
        let mut file = File::create(file_path)?;
        file.write_all(serialized.as_bytes())?;
    }
    Ok(())
}

fn load_cebra_settings_with_dialog() -> Result<Vec<Cebr3Detector>, Box<dyn std::error::Error>> {
    if let Some(file_path) = FileDialog::new()
        .add_filter("YAML Files", &["yaml", "yml"])  // Add a filter for YAML files
        .pick_file() {
            
        let data = read_to_string(file_path)?;
        let detectors: Vec<Cebr3Detector> = serde_yaml::from_str(&data)?;
        return Ok(detectors);
    }
    Err("No file selected".into())
}

pub fn add_cebra_histograms(file_paths: Arc<[PathBuf]>, detectors: &[Cebr3Detector]) -> Result<Histogrammer, PolarsError> {

    let args = ScanArgsParquet::default();
    let lf = LazyFrame::scan_parquet_files(file_paths, args)?;

    let mut h = Histogrammer::new();

    // Use the actual detectors here

    let cebra_ecal_range = (0.0, 6000.0);
    let cebra_ecal_bins = 500;

    h.add_hist1d("CeBrAEnergyGainMatched", 512, (0.0, 4096.0));
    h.add_hist1d("CeBrAEnergyCalibrated", cebra_ecal_bins, cebra_ecal_range);

    for detector in detectors {
        let num = detector.number;

        let gain_m = detector.gain_matched_values[0]; // Extract m
        let gain_b = detector.gain_matched_values[1]; // Extract b

        
        let ecal_a = detector.energy_calibration_values[0]; // Extract a
        let ecal_b = detector.energy_calibration_values[1]; // Extract b
        let ecal_c = detector.energy_calibration_values[2]; // Extract c

        let lf = lf.clone().with_column(
            (col(&format!("Cebra{}Energy", num)) * lit(gain_m) + lit(gain_b))
            .alias(&format!("Cebra{}EnergyGainMatched", num))
        );

        let lf = lf.clone().with_column(
            ( col(&format!("Cebra{}EnergyGainMatched", num)) * col(&format!("Cebra{}EnergyGainMatched", num)) * lit(ecal_a)
            + col(&format!("Cebra{}EnergyGainMatched", num)) * lit(ecal_b)
            + lit(ecal_c) )
            .alias(&format!("Cebra{}EnergyCalibrated", num))
        );

        h.add_fill_hist1d_from_polars(&format!("Cebra{}Energy", num), &lf, &format!("Cebra{}Energy", num), 512, (0.0, 4096.0));
        h.add_fill_hist1d_from_polars(&format!("Cebra{}EnergyGainMatched", num), &lf, &format!("Cebra{}EnergyGainMatched", num), 512, (0.0, 4096.0));
        h.add_fill_hist1d_from_polars(&format!("Cebra{}EnergyCalibrated", num), &lf, &format!("Cebra{}EnergyCalibrated", num), cebra_ecal_bins, cebra_ecal_range);

        h.fill_hist1d_from_polars("CeBrAEnergyGainMatched", &lf, &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist1d_from_polars("CeBrAEnergyCalibrated", &lf, &format!("Cebra{}EnergyCalibrated", num));

    }

    Ok(h)
}
