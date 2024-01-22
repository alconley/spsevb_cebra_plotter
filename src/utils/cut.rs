use polygonical::polygon::Polygon;
use polygonical::point::Point;

use polars::prelude::*;

use crate::utils::egui_polygon::EditableEguiPolygon;

// typical cut names for sps experiments
const CUT_COLUMN_NAMES: &[&str] = &["AnodeBackEnergy", "AnodeFrontEnergy", "Cathode", "ScintLeftEnergy", "Xavg", "X1", "X2"];

pub struct Cut2D {
    polygon: Polygon,
}

impl Cut2D {
    // Create a new Cut2D from an EditableEguiPolygon
    pub fn new(editable_polygon: EditableEguiPolygon) -> Self {

        // converts the EditableEguiPolygon to a Polygon
        let points = editable_polygon.vertices.iter().map(|v| Point { x: v[0], y: v[1] }).collect();

        let polygon = Polygon::new(points);
        Self { polygon }
    }

    // Function to create a mask from a Polars DataFrame
    pub fn filter_dataframe(&self, dataframe: &LazyFrame, x_column_name: &str, y_column_name: &str) -> Result<LazyFrame, polars::error::PolarsError> {

        let df = dataframe.clone()
            // .select([col(x_column_name), col(y_column_name)])
            .filter(col(x_column_name).neq(lit(-1e6)))
            .filter(col(y_column_name).neq(lit(-1e6)))
            .collect()?;

        let x_col = df.column(x_column_name)?;
        let y_col = df.column(y_column_name)?;

        let mask = x_col.f64()?
            .into_iter()
            .zip(y_col.f64()?)
            .map(|(x, y)| {
                match (x, y) {
                    (Some(x), Some(y)) => {
                        let point = Point { x, y };
                        self.polygon.contains(point)
                    },
                    _ => false,
                }
            })
            .collect::<BooleanChunked>();

        let filtered_df = df.filter(&mask)?.lazy();

        Ok(filtered_df)
    }



}

pub struct CutHandler {
    pub current_editable_polygon: Option<EditableEguiPolygon>,
}

impl CutHandler {
    pub fn new() -> Self {
        Self {
            current_editable_polygon: None,
        }
    }

    pub fn cut_handler_ui(&mut self, ui: &mut egui::Ui) {

        ui.horizontal(|ui| {
            ui.label("Cutter");
            ui.separator();

            if ui.button("New").clicked() {
                self.create_new_cut();
            }

            ui.separator();

            if ui.button("Load").clicked() {
                // Create a new EditableEguiPolygon to load the data into
                let mut new_polygon = EditableEguiPolygon::new();
                if let Err(e) = new_polygon.load_cut_from_json() {
                    // Handle the error, e.g., show an error message
                    eprintln!("Error loading cut: {:?}", e);
                } else {
                    // Successfully loaded, update current_editable_polygon
                    self.current_editable_polygon = Some(new_polygon);
                }
            }

            if let Some(editable_polygon) = self.current_editable_polygon.as_mut() {

                let combo_box_width = 125.0;
                    // X Column ComboBox
                    egui::ComboBox::from_label("X Column")
                    .selected_text(editable_polygon.selected_x_column.as_deref().unwrap_or(""))
                    .width(combo_box_width)
                    .show_ui(ui, |ui| {
                        for &column in CUT_COLUMN_NAMES.iter() {
                            if ui.selectable_label(editable_polygon.selected_x_column.as_deref() == Some(column), column).clicked() {
                                editable_polygon.selected_x_column = Some(column.to_string());
                            }
                        }
                    });
        
                    // Y Column ComboBox
                    egui::ComboBox::from_label("Y Column")
                        .selected_text(editable_polygon.selected_y_column.as_deref().unwrap_or(""))
                        .width(combo_box_width)
                        .show_ui(ui, |ui| {
                            for &column in CUT_COLUMN_NAMES.iter() {
                                if ui.selectable_label(editable_polygon.selected_y_column.as_deref() == Some(column), column).clicked() {
                                    editable_polygon.selected_y_column = Some(column.to_string());
                                }
                            }
                        });

                ui.separator();

                // Check if both X and Y columns are selected
                let can_save: bool = editable_polygon.selected_x_column.is_some() && editable_polygon.selected_y_column.is_some();

                // "Save Cut" button is enabled only if both columns are selected
                if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                    if let Err(e) = editable_polygon.save_cut_to_json() {
                        eprintln!("Error saving cut: {:?}", e);
                    }
                }

                ui.separator();

                if ui.button("Remove Cut").clicked() {
                    // Remove the current editable polygon
                    self.current_editable_polygon = None;
                }

            }

        });
    }

    // Method to start creating a new cut
    fn create_new_cut(&mut self) {
            self.current_editable_polygon = Some(EditableEguiPolygon::new());
        }


}
