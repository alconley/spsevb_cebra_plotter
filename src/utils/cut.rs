use crate::utils::egui_polygon::EditableEguiPolygon;
use std::collections::HashMap;
use egui_plot::PlotUi;

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
    pub fn cut_handler_ui(&mut self, ui: &mut egui::Ui) {
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

    // pub fn filter_lf_with_cuts(&mut self, lf: &LazyFrame) -> Vec<PathBuf> {
    //     let mut filtered_files = Vec::new();

    //     filtered_files
    // }

}
