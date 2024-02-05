use crate::utils::egui_polygon::EditableEguiPolygon;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;

use rfd::FileDialog;
use egui_plot::PlotUi;
use polars::prelude::*;

pub struct CutHandler {
    pub cuts: HashMap<String, EditableEguiPolygon>,
    pub active_cut_id: Option<String>,
    pub draw_flag: bool,
}

impl CutHandler {
    // Creates a new `CutHandler` instance.
    pub fn new() -> Self {
        Self {
            cuts: HashMap::new(),
            active_cut_id: None,
            draw_flag: true,
        }
    }

    // Adds a new cut and makes it the active one
    pub fn add_new_cut(&mut self) {
        let new_id = format!("cut_{}", self.cuts.len() + 1);
        self.cuts.insert(new_id.clone(), EditableEguiPolygon::new());
        self.active_cut_id = Some(new_id); // Automatically make the new cut active
    }

    // UI handler for the cut handler.
    pub fn cut_handler_ui(&mut self, ui: &mut egui::Ui, file_paths: Arc<[PathBuf]>) {
        ui.horizontal(|ui| {
            ui.label("2D Cutter");
            ui.separator();

            if ui.button("New").clicked() {
                self.add_new_cut();
            }

            ui.separator();

            // remove active cut
            if let Some(active_id) = &self.active_cut_id {
                if ui.button("Remove Active Cut").clicked() {
                    self.cuts.remove(active_id);
                    self.active_cut_id = None;
                }
            }

            ui.separator();

            if ui.button("Save Cut Reduced DataFrame").clicked() {
                // Open a save file dialog
                if let Some(path) = FileDialog::new()
                    .set_title("Save Reduced DataFrame")
                    .add_filter("Parquet file", &["parquet"])
                    .save_file() {
                        // Convert PathBuf from dialog to a format suitable for your function
                        let output_path = path;
    
                        // Here, you should call your filtering and saving method
                        // Assuming this method exists and works with the provided file_paths and output_path
                        if let Err(e) = self.filter_files_with_cuts_and_save(file_paths.clone(), &output_path) {
                            eprintln!("Failed to save DataFrame: {:?}", e);
                        }
                }
            }


        });

        ui.horizontal(|ui| {

            // If there are cuts, display a ComboBox to select the active cut
            if !self.cuts.is_empty() {
                let selected_label = self.active_cut_id.clone().unwrap_or_else(|| "Select a cut".to_string());
                egui::ComboBox::from_label("Active Cut")
                    .selected_text(&selected_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.active_cut_id, None, "None"); // Option to deselect any active cut
                        for (id, _) in self.cuts.iter() {
                            let label = format!("{}", id);
                            ui.selectable_value(&mut self.active_cut_id, Some(id.clone()), &label);
                        }
                    });
            }

            // Display UI for the active cut
            if let Some(active_id) = &self.active_cut_id {
                if let Some(active_cut) = self.cuts.get_mut(active_id) {
                    // ui.add_space(10.0); // Add some space before the active cut UI
                    active_cut.cut_ui(ui);
                }

                ui.separator();

                ui.checkbox(&mut self.draw_flag, "Draw");
            }

        });
    }

    // Method to draw the active cut
    pub fn draw_active_cut(&mut self, plot_ui: &mut PlotUi) {
        if self.draw_flag {
            if let Some(active_id) = &self.active_cut_id {
                if let Some(active_cut) = self.cuts.get_mut(active_id) {
                    active_cut.draw(plot_ui);
                }
            }
        }
    }


    pub fn filter_files_with_cuts_and_save(&mut self, file_paths: Arc<[PathBuf]>, output_path: &PathBuf) -> Result<(), PolarsError> {
        let args = ScanArgsParquet::default();

        // Assuming LazyFrame::scan_parquet_files constructs a LazyFrame from the list of files
        let lf = LazyFrame::scan_parquet_files(file_paths, args)?;

        // Apply filtering logic as before, leading to a filtered LazyFrame
        let filtered_lf = self.filter_lf_with_cuts(&lf)?; // Placeholder for applying cuts

        // Collect the LazyFrame into a DataFrame
        let mut filtered_df = filtered_lf.collect()?;

        // Open a file in write mode at the specified output path
        let file = File::create(output_path)
            .map_err(|e| PolarsError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Write the filtered DataFrame to a Parquet file
        ParquetWriter::new(file)
            .finish(&mut filtered_df)?;

        Ok(())
    }

    pub fn filter_lf_with_cuts(&mut self, lf: &LazyFrame) -> Result<LazyFrame, PolarsError> {

        // this is a lot of work to filter the lazy frame with the cuts but it works

        let filtered_lf = lf.clone();

        // Iterate through the cuts, get column names, and filter the lazy frame with the null values (-1e6) first before collecting
        for (_id, cut) in self.cuts.iter() {
            if let (Some(x_col_name), Some(y_col_name)) = (&cut.selected_x_column, &cut.selected_y_column) {
                let filtered_lf = filtered_lf.clone()
                    .filter(col(x_col_name).neq(lit(-1e6)))
                    .filter(col(y_col_name).neq(lit(-1e6)));
            }
        }

        // Vector to store the masks for each cut
        let mut masks: Vec<Vec<bool>> = Vec::new();

        // Iterate through the cuts, get column names, collect columns, convert to ndarray, 
        // check if the point is inside the polygon, and then create a mask
        for (_id, cut) in self.cuts.iter() {

            if let (Some(x_col_name), Some(y_col_name)) = (&cut.selected_x_column, &cut.selected_y_column) {
                let mask_creation_df = filtered_lf.clone()
                            .select([col(x_col_name), col(y_col_name)])
                            .collect()?;

                let ndarray_mask_creation_df = mask_creation_df.to_ndarray::<Float64Type>(IndexOrder::Fortran)?;
                
                let shape = ndarray_mask_creation_df.shape();
                let rows = shape[0];

                let mut mask: Vec<bool> = Vec::new();

                // Iterating through the ndarray rows and check if the point is inside the polygon
                for i in 0..rows {
                    let x_value = ndarray_mask_creation_df[[i, 0]];
                    let y_value = ndarray_mask_creation_df[[i, 1]];

                    let point = cut.is_inside(x_value, y_value);
                    mask.push(point);
                }

                masks.push(mask);

            }

        }

        // Initialize the final combined mask with false values
        // Assume all masks are of equal length, and `dataset_len` is the length of your dataset
        let dataset_len = masks.first().map_or(0, |m| m.len());
        let mut combined_mask = vec![false; dataset_len];

        // Iterate through each mask and combine it with the combined_mask using logical OR
        for mask in masks {
            for (i, &value) in mask.iter().enumerate() {
                combined_mask[i] = combined_mask[i] || value;
            }
        }

        // Convert the combined_mask Vec<bool> to BooleanChunked for filtering
        let mut boolean_chunked_builder = BooleanChunkedBuilder::new("combined_mask", combined_mask.len());
        for &value in &combined_mask {
            boolean_chunked_builder.append_value(value);
        }
        let boolean_chunked_series = boolean_chunked_builder.finish();
        
        // collect the filtered lazy frame
        let filtered_df = filtered_lf.collect()?;

        // filter filtered_df with the combined_mask and convert to lazy frame
        let cuts_filtered_lf = filtered_df.filter(&boolean_chunked_series)?.lazy();

        Ok(cuts_filtered_lf)
    }

}
