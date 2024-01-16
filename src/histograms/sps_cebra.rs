// Standard library imports
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::f32::consts::PI;


// External crates imports
use egui::Ui;
use polars::prelude::*;
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use serde_yaml;

// Local crate/module imports
use crate::utils::histogrammer::{Histogrammer};

#[derive(Serialize, Deserialize)]
pub struct Cebr3DetectorWithSPS {
    number: i32,
    gain_matched_values: [f64; 2],  // Tuple for 'm' and 'b'
    energy_calibration_values: [f64; 3],  // Tuple for 'a', 'b', and 'c'
    time_gate: [f64; 3],  // Tuple for left, right, and shift value for the CebraTime-ScintLeftTime histogram
}

pub fn sps_cebra_detector_ui(detectors: &mut Vec<Cebr3DetectorWithSPS>, ui: &mut Ui) {
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

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Time Gate:");
                ui.add(egui::DragValue::new(&mut detector.time_gate[0]).max_decimals(10).speed(0.1).prefix("Left Gate: "));
                ui.add(egui::DragValue::new(&mut detector.time_gate[1]).max_decimals(10).speed(0.1).prefix("Right Gate: "));  
                ui.add(egui::DragValue::new(&mut detector.time_gate[2]).max_decimals(10).speed(0.1).prefix("Shift Value: ")); 
            });

        });

    }

    ui.horizontal(|ui| {

        let max_det_number = detectors.iter().map(|d| d.number).max().unwrap_or(-1);

        // Check if the maximum number is less than 6 before showing the button
        if max_det_number < 6 {
            if ui.button("Add Detector").clicked() {
                detectors.push(Cebr3DetectorWithSPS {
                    number: max_det_number + 1, // Increment the maximum number
                    gain_matched_values: [1.0, 0.0],
                    energy_calibration_values: [0.0, 1.0, 0.0],
                    time_gate: [-3000.0, 3000.0, 0.0],
                });
            }
        } else {
            // Optional: show a disabled button or a label explaining why adding more is not possible
            ui.add_enabled(false, egui::Button::new("Add Detector"));
        }

        ui.separator();

        if ui.button("Save Calibration Settings").clicked() {
            if let Err(e) = save_sps_cebra_settings_with_dialog(detectors) {
                eprintln!("Failed to save detectors: {}", e);
            }
        }

        ui.separator();

        if ui.button("Load Calibration Settings").clicked() {
            match load_sps_cebra_settings_with_dialog() {
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

fn save_sps_cebra_settings_with_dialog(detectors: &[Cebr3DetectorWithSPS]) -> Result<(), Box<dyn std::error::Error>> {
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

fn load_sps_cebra_settings_with_dialog() -> Result<Vec<Cebr3DetectorWithSPS>, Box<dyn std::error::Error>> {
    if let Some(file_path) = FileDialog::new()
        .add_filter("YAML Files", &["yaml", "yml"])
        .pick_file() {
            
        let data = read_to_string(file_path)?;
        let detectors: Vec<Cebr3DetectorWithSPS> = serde_yaml::from_str(&data)?;
        return Ok(detectors);
    }
    Err("No file selected".into())
}

pub fn add_sps_cebra_histograms(file_paths: Arc<[PathBuf]>, detectors: &[Cebr3DetectorWithSPS]) -> Result<Histogrammer, PolarsError> {

        let args = ScanArgsParquet::default();

        // Load multiple parquet files
        let lf = LazyFrame::scan_parquet_files(file_paths, args)?;

        let mut h = Histogrammer::new();

    let lf = lf.with_columns(vec![
        (col("DelayFrontRightEnergy")+col("DelayFrontLeftEnergy")/ lit(2.0) ).alias("DelayFrontAverageEnergy"),
        (col("DelayBackRightEnergy")+col("DelayBackLeftEnergy")/ lit(2.0) ).alias("DelayBackAverageEnergy"),
        (col("DelayFrontLeftTime") - col("AnodeFrontTime")).alias("DelayFrontLeftTime_AnodeFrontTime"),
        (col("DelayFrontRightTime") - col("AnodeFrontTime")).alias("DelayFrontRightTime_AnodeFrontTime"),
        (col("DelayBackLeftTime") - col("AnodeFrontTime")).alias("DelayBackLeftTime_AnodeFrontTime"),
        (col("DelayBackRightTime") - col("AnodeFrontTime")).alias("DelayBackRightTime_AnodeFrontTime"),
        (col("DelayFrontLeftTime") - col("AnodeBackTime")).alias("DelayFrontLeftTime_AnodeBackTime"),
        (col("DelayFrontRightTime") - col("AnodeBackTime")).alias("DelayFrontRightTime_AnodeBackTime"),
        (col("DelayBackLeftTime") - col("AnodeBackTime")).alias("DelayBackLeftTime_AnodeBackTime"),
        (col("DelayBackRightTime") - col("AnodeBackTime")).alias("DelayBackRightTime_AnodeBackTime"),
        (col("AnodeFrontTime") - col("AnodeBackTime")).alias("AnodeFrontTime_AnodeBackTime"),
        (col("AnodeBackTime") - col("AnodeFrontTime")).alias("AnodeBackTime_AnodeFrontTime"),
        (col("AnodeFrontTime") - col("ScintLeftTime")).alias("AnodeFrontTime_ScintLeftTime"),
        (col("AnodeBackTime") - col("ScintLeftTime")).alias("AnodeBackTime_ScintLeftTime"),
        (col("DelayFrontLeftTime") - col("ScintLeftTime")).alias("DelayFrontLeftTime_ScintLeftTime"),
        (col("DelayFrontRightTime") - col("ScintLeftTime")).alias("DelayFrontRightTime_ScintLeftTime"),
        (col("DelayBackLeftTime") - col("ScintLeftTime")).alias("DelayBackLeftTime_ScintLeftTime"),
        (col("DelayBackRightTime") - col("ScintLeftTime")).alias("DelayBackRightTime_ScintLeftTime"),
        (col("ScintRightTime") - col("ScintLeftTime")).alias("ScintRightTime_ScintLeftTime"),
    ]);

    h.add_fill_hist1d_from_polars("X1", &lf, "X1", 600, (-300.0, 300.0));
    h.add_fill_hist1d_from_polars("X2", &lf, "X2", 600, (-300.0, 300.0));
    h.add_fill_hist2d_from_polars("X2_X1", &lf, "X1", 600, (-300.0, 300.0), "X2", 600, (-300.0,300.0));
    h.add_fill_hist2d_from_polars("DelayBackRight_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayBackRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackLeft_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayBackLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontRight_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayFrontRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontLeft_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayFrontLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackRight_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayBackRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackLeft_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayBackLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontRight_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayFrontRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontLeft_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayFrontLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackRight_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayBackRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackLeft_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayBackLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontRight_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayFrontRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontLeft_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayFrontLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontAverage_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayFrontAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackAverage_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayBackAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontAverage_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayFrontAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackAverage_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayBackAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontAverage_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayFrontAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackAverage_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayBackAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_ScintLeft", &lf, "ScintLeftEnergy", 256, (0.0, 4096.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_ScintLeft", &lf, "ScintLeftEnergy", 256, (0.0, 4096.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_ScintLeft", &lf, "ScintLeftEnergy", 256, (0.0, 4096.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_ScintRight", &lf, "ScintRightEnergy", 256, (0.0, 4096.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_ScintRight", &lf, "ScintRightEnergy", 256, (0.0, 4096.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_ScintRight", &lf, "ScintRightEnergy", 256, (0.0, 4096.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintLeft_X1", &lf, "X1", 600, (-300.0, 300.0), "ScintLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintLeft_X2", &lf, "X2", 600, (-300.0, 300.0), "ScintLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintLeft_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "ScintLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintRight_X1", &lf, "X1", 600, (-300.0, 300.0), "ScintRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintRight_X2", &lf, "X2", 600, (-300.0, 300.0), "ScintRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintRight_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "ScintRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_X1", &lf, "X1", 600, (-300.0, 300.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_X2", &lf, "X2", 600, (-300.0, 300.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_X1", &lf, "X1", 600, (-300.0, 300.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_X2", &lf, "X2", 600, (-300.0, 300.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_X1", &lf, "X1", 600, (-300.0, 300.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_X2", &lf, "X2", 600, (-300.0, 300.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "CathodeEnergy", 256, (0.0, 4096.0));

    // Both planes histograms
    let lf_bothplanes = lf.clone().filter(col("X1").neq(lit(-1e6))).filter(col("X2").neq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("X1_bothplanes", &lf_bothplanes, "X1", 600, (-300.0, 300.0));
    h.add_fill_hist1d_from_polars("X2_bothplanes", &lf_bothplanes, "X2", 600, (-300.0, 300.0));
    h.add_fill_hist1d_from_polars("Xavg_bothplanes", &lf_bothplanes, "Xavg", 600, (-300.0, 300.0));

    h.add_fill_hist2d_from_polars("Theta_Xavg_bothplanes", &lf_bothplanes, "Xavg", 600, (-300.0, 300.0), "Theta", 300, (0.0, (PI/2.0).into()));
    h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeFrontTime_bothplanes", &lf_bothplanes, "DelayFrontLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeFrontTime_bothplanes", &lf_bothplanes, "DelayFrontRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeBackTime_bothplanes", &lf_bothplanes, "DelayBackLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeBackTime_bothplanes", &lf_bothplanes, "DelayBackRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    
    // Only 1 plane: X1
    let lf_only_x1_plane = lf.clone().filter(col("X1").neq(lit(-1e6))).filter(col("X2").eq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("X1_only1plane", &lf_only_x1_plane, "X1", 600, (-300.0, 300.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayFrontLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayFrontRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayBackLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayBackRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayFrontLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayFrontRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayBackLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayBackRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));

    // Only 1 plane: X2
    let lf_only_x2_plane = lf.clone().filter(col("X2").neq(lit(-1e6))).filter(col("X1").eq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("X2_only1plane", &lf_only_x2_plane, "X2", 600, (-300.0, 300.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayFrontLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayFrontRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayBackLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayBackRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayFrontLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayFrontRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayBackLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayBackRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));

    // Time relative to Back Anode

    let lf_time_rel_backanode = lf.clone().filter(col("AnodeBackTime").neq(lit(-1e6))).filter(col("ScintLeftTime").neq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("AnodeFrontTime_AnodeBackTime", &lf_time_rel_backanode, "AnodeFrontTime_AnodeBackTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("AnodeBackTime_AnodeFrontTime", &lf_time_rel_backanode, "AnodeBackTime_AnodeFrontTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("AnodeFrontTime_ScintLeftTime", &lf_time_rel_backanode, "AnodeFrontTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("AnodeBackTime_ScintLeftTime", &lf_time_rel_backanode, "AnodeBackTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayFrontLeftTime_ScintLeftTime", &lf_time_rel_backanode, "DelayFrontLeftTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayFrontRightTime_ScintLeftTime", &lf_time_rel_backanode, "DelayFrontRightTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayBackLeftTime_ScintLeftTime", &lf_time_rel_backanode, "DelayBackLeftTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayBackRightTime_ScintLeftTime", &lf_time_rel_backanode, "DelayBackRightTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("ScintRightTime_ScintLeftTime", &lf_time_rel_backanode, "ScintRightTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist2d_from_polars("ScintTimeDif_Xavg", &lf_time_rel_backanode, "Xavg", 600, (-300.0, 300.0), "ScintRightTime_ScintLeftTime", 12800, (-3200.0, 3200.0));

    // Histograms that are related to CeBrA

    let cebra_ecal_range = (0.0, 6000.0);
    let cebra_ecal_bins = 500;

    // summed with no time cuts
    h.add_hist1d("CeBrAEnergyGainMatched", 512, (0.0, 4096.0));
    h.add_hist1d("CeBrAEnergyCalibrated", cebra_ecal_bins, cebra_ecal_range);
    h.add_hist1d("CeBrATimeToScintShifted", 6000, (-3000.0, 3000.0));

    h.add_hist2d("CeBrAEnergyGainMatched_Xavg", 600, (-300.0, 300.0), 512, (0.0, 4096.0));
    h.add_hist2d("CeBrAEnergyCalibrated_Xavg", 600, (-300.0, 300.0), cebra_ecal_bins, cebra_ecal_range);

    h.add_hist2d("CeBrAEnergyGainMatched_X1", 600, (-300.0, 300.0), 512, (0.0, 4096.0));
    h.add_hist2d("CeBrAEnergyCalibrated_X1", 600, (-300.0, 300.0), cebra_ecal_bins, cebra_ecal_range);

    h.add_hist2d("CeBrATimeToScintShifted_Xavg", 600, (-300.0, 300.0), 100, (-50.0, 50.0));

    // summed with time cuts
    h.add_hist1d("CeBrAEnergyGainMatched_TimeCut", 512, (0.0, 4096.0));
    h.add_hist1d("CeBrAEnergyCalibrated_TimeCut", cebra_ecal_bins, cebra_ecal_range);
    h.add_hist1d("CeBrATimeToScint_TimeCut", 100, (-50.0, 50.0));

    h.add_hist2d("CeBrAEnergyGainMatched_Xavg_TimeCut", 600, (-300.0, 300.0), 512, (0.0, 4096.0));
    h.add_hist2d("CeBrAEnergyCalibrated_Xavg_TimeCut", 600, (-300.0, 300.0), cebra_ecal_bins, cebra_ecal_range);
    h.add_hist2d("CeBrAEnergyGainMatched_X1_TimeCut", 600, (-300.0, 300.0), 512, (0.0, 4096.0));
    h.add_hist2d("CeBrAEnergyCalibrated_X1_TimeCut", 600, (-300.0, 300.0), cebra_ecal_bins, cebra_ecal_range);

    h.add_hist1d("Xavg_TimeCut", 600, (-300.0, 300.0));

    for detector in detectors {


        let det_lf = lf.clone().filter(col(&format!("Cebra{}Energy", detector.number)).neq(lit(-1e6)));

        let num = detector.number;

        let gain_m = detector.gain_matched_values[0]; // Extract m
        let gain_b = detector.gain_matched_values[1]; // Extract b

        let time_gate_left = detector.time_gate[0]; // Extract left time gate
        let time_gate_right = detector.time_gate[1]; // Extract right time gate
        let time_gate_shift = detector.time_gate[2]; // Extract shift value

        let det_lf = det_lf.with_columns(vec![
            (col(&format!("Cebra{}Time", num)) - col("ScintLeftTime")).alias(&format!("Cebra{}TimeToScint", num)),
            (col(&format!("Cebra{}Time", num)) - col("ScintLeftTime") + lit(time_gate_shift)).alias(&format!("Cebra{}TimeToScintShifted", num)),
            (col(&format!("Cebra{}Energy", num)) * lit(gain_m) + lit(gain_b)).alias(&format!("Cebra{}EnergyGainMatched", num))
        ]);

        let ecal_a = detector.energy_calibration_values[0]; // Extract a
        let ecal_b = detector.energy_calibration_values[1]; // Extract b
        let ecal_c = detector.energy_calibration_values[2]; // Extract c

        let det_lf = det_lf.clone().with_column(
            ( col(&format!("Cebra{}EnergyGainMatched", num)) * col(&format!("Cebra{}EnergyGainMatched", num)) * lit(ecal_a)
            + col(&format!("Cebra{}EnergyGainMatched", num)) * lit(ecal_b)
            + lit(ecal_c) )
            .alias(&format!("Cebra{}EnergyCalibrated", num))
        );

        h.add_fill_hist1d_from_polars(&format!("Cebra{}Energy", num), &det_lf, &format!("Cebra{}Energy", num), 512, (0.0, 4096.0));
        h.add_fill_hist1d_from_polars(&format!("Cebra{}EnergyGainMatched", num), &det_lf, &format!("Cebra{}EnergyGainMatched", num), 512, (0.0, 4096.0));
        h.add_fill_hist1d_from_polars(&format!("Cebra{}EnergyCalibrated", num), &det_lf, &format!("Cebra{}EnergyCalibrated", num), cebra_ecal_bins, cebra_ecal_range);

        let det_time_lf = det_lf.clone().filter(col("ScintLeftEnergy").neq(lit(-1e6))).filter(col("AnodeBackEnergy").neq(lit(-1e6)));

        h.add_fill_hist1d_from_polars(&format!("Cebra{}TimeToScint", num), &det_time_lf, &format!("Cebra{}TimeToScint", num), 6000, (-3000.0, 3000.0));

        // summed plots with no time cuts
        h.fill_hist1d_from_polars("CeBrAEnergyGainMatched", &det_lf, &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist1d_from_polars("CeBrAEnergyCalibrated", &det_lf, &format!("Cebra{}EnergyCalibrated", num));

        h.fill_hist2d_from_polars("CeBrAEnergyGainMatched_Xavg", &det_lf, "Xavg", &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist2d_from_polars("CeBrAEnergyCalibrated_Xavg", &det_lf, "Xavg", &format!("Cebra{}EnergyCalibrated", num));
        h.fill_hist2d_from_polars("CeBrAEnergyGainMatched_X1", &det_lf, "X1", &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist2d_from_polars("CeBrAEnergyCalibrated_X1", &det_lf, "X1", &format!("Cebra{}EnergyCalibrated", num));

        h.fill_hist1d_from_polars("CeBrATimeToScintShifted", &det_time_lf, &format!("Cebra{}TimeToScintShifted", num));
        h.fill_hist2d_from_polars("CeBrATimeToScintShifted_Xavg", &det_time_lf, "Xavg", &format!("Cebra{}TimeToScintShifted", num));

        // time cuts
        let det_tcut_lf = det_lf
            .filter(col("ScintLeftEnergy").neq(lit(-1e6)))
            .filter(col("AnodeBackEnergy").neq(lit(-1e6)))
            .filter(col(&format!("Cebra{}TimeToScint", num)).gt(lit(time_gate_left)))
            .filter(col(&format!("Cebra{}TimeToScint", num)).lt(lit(time_gate_right)));

        h.add_fill_hist1d_from_polars(&format!("Cebra{}Energy_TimeCut", num), &det_tcut_lf, &format!("Cebra{}Energy", num), 512, (0.0, 4096.0));
        h.add_fill_hist1d_from_polars(&format!("Cebra{}EnergyGainMatched_TimeCut", num), &det_tcut_lf, &format!("Cebra{}EnergyGainMatched", num), 512, (0.0, 4096.0));
        h.add_fill_hist1d_from_polars(&format!("Cebra{}EnergyCalibrated_TimeCut", num), &det_tcut_lf, &format!("Cebra{}EnergyCalibrated", num), cebra_ecal_bins, cebra_ecal_range);
        h.add_fill_hist1d_from_polars(&format!("Cebra{}TimeToScint_TimeCut", num), &det_tcut_lf, &format!("Cebra{}TimeToScint", num), 6000, (-3000.0, 3000.0));

        // summed plots with time cuts
        h.fill_hist1d_from_polars("CeBrAEnergyGainMatched_TimeCut", &det_tcut_lf, &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist1d_from_polars("CeBrAEnergyCalibrated_TimeCut", &det_tcut_lf, &format!("Cebra{}EnergyCalibrated", num));
        h.fill_hist1d_from_polars("CeBrATimeToScint_TimeCut", &det_tcut_lf, &format!("Cebra{}TimeToScintShifted", num));
        h.fill_hist1d_from_polars("Xavg_TimeCut", &det_tcut_lf, "Xavg");
        h.fill_hist2d_from_polars("CeBrAEnergyGainMatched_Xavg_TimeCut", &det_tcut_lf, "Xavg", &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist2d_from_polars("CeBrAEnergyCalibrated_Xavg_TimeCut", &det_tcut_lf, "Xavg", &format!("Cebra{}EnergyCalibrated", num));
        h.fill_hist2d_from_polars("CeBrAEnergyGainMatched_X1_TimeCut", &det_tcut_lf, "X1", &format!("Cebra{}EnergyGainMatched", num));
        h.fill_hist2d_from_polars("CeBrAEnergyCalibrated_X1_TimeCut", &det_tcut_lf, "X1", &format!("Cebra{}EnergyCalibrated", num));

    
    }

    Ok(h)
}