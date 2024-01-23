use crate::utils::egui_polygon::EditableEguiPolygon;

// typical cut names for sps experiments
const CUT_COLUMN_NAMES: &[&str] = &[
    "AnodeBackEnergy", "AnodeFrontEnergy", "Cathode",
     "ScintLeftEnergy", "Xavg", "X1", "X2"
];

pub struct CutHandler {
    pub current_editable_polygon: Option<EditableEguiPolygon>,
}

impl CutHandler {
    // Creates a new `CutHandler` instance.
    pub fn new() -> Self {
        Self {
            current_editable_polygon: None,
        }
    }

    // UI handler for the cut handler.
    pub fn cut_handler_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Cutter");
            ui.separator();

            if ui.button("New").clicked() {
                self.create_new_cut();
            }

            ui.separator();

            if ui.button("Load").clicked() {
                let mut new_polygon = EditableEguiPolygon::new();
                if let Err(e) = new_polygon.load_cut_from_json() {
                    eprintln!("Error loading cut: {:?}", e);
                } else {
                    self.current_editable_polygon = Some(new_polygon);
                }
            }

            // Show buttons if there is a current editable polygon
            if let Some(editable_polygon) = self.current_editable_polygon.as_mut() {
                // X Column ComboBox
                egui::ComboBox::from_label("X Column")
                    .selected_text(editable_polygon.selected_x_column.as_deref().unwrap_or(""))
                    .width(125.0)
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
                    .width(125.0)
                    .show_ui(ui, |ui| {
                        for &column in CUT_COLUMN_NAMES.iter() {
                            if ui.selectable_label(editable_polygon.selected_y_column.as_deref() == Some(column), column).clicked() {
                                editable_polygon.selected_y_column = Some(column.to_string());
                            }
                        }
                    });

                ui.separator();

                // Save Cut button
                let can_save: bool = editable_polygon.selected_x_column.is_some() && editable_polygon.selected_y_column.is_some();
                if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                    if let Err(e) = editable_polygon.save_cut_to_json() {
                        eprintln!("Error saving cut: {:?}", e);
                    }
                }

                ui.separator();

                // Remove Cut button
                if ui.button("Remove Cut").clicked() {
                    // Remove the current editable polygon
                    self.current_editable_polygon = None;
                }
            }

        });
    }


    // Creates a new editable polygon.
    fn create_new_cut(&mut self) {
            self.current_editable_polygon = Some(EditableEguiPolygon::new());
        }


}
