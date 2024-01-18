use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::collections::HashMap;

use egui_plot::{Points, PlotPoints, Polygon, PlotUi};
use eframe::egui::{Color32, Stroke};

use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use serde_json;

// typical cut names for sps experiments
const CUT_COLUMN_NAMES: &[&str] = &["AnodeBackEnergy", "AnodeFrontEnergy", "Cathode", "ScintLeftEnergy", "Xavg", "X1", "X2"];


#[derive(Default, Serialize, Deserialize)]
pub struct EditablePolygon {
    vertices: Vec<[f64; 2]>,        // List of vertex coordinates
    selected_vertex_index: Option<usize>,  // Index of the selected vertex (if any)
    selected_x_column: Option<String>,
    selected_y_column: Option<String>,
}

impl EditablePolygon {
    /// Creates a new `EditablePolygon` with default vertices.
    /// Current Cut Binds: 
    ///     Right click to add verticies 
    ///     Left click to remove verticies
    ///     Middle click to remove all verticies
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),  // Initialize with an empty set of vertices
            selected_vertex_index: None,  // Initially, no vertex is selected
            selected_x_column: None,
            selected_y_column: None,
        }
    }

    pub fn cut_ui(&mut self, ui: &mut egui::Ui, draw_cut: &mut bool) {
        ui.horizontal(|ui| {
            ui.label("Cut");
            ui.separator();

            if ui.button("Load Cut").clicked() {
                if let Err(e) = self.load_cut_from_json() {
                    eprintln!("Failed to load cut: {:?}", e);
                }
                *draw_cut = true;
            }
    
            // Checkbox to toggle the display of additional UI elements
            ui.checkbox(draw_cut, "Draw Cut");
    
            // Conditional display based on the checkbox state
            if *draw_cut {
                ui.separator();
    
                // Dropdown for selecting the X column
                egui::ComboBox::from_label(": X")
                    .selected_text(self.selected_x_column.as_deref().unwrap_or("Select X Column"))
                    .show_ui(ui, |ui| {
                        for &name in CUT_COLUMN_NAMES {
                            if ui.selectable_label(self.selected_x_column.as_deref() == Some(name), name).clicked() {
                                self.selected_x_column = Some(name.to_string());
                            }
                        }
                    });
    
                // Dropdown for selecting the Y column
                egui::ComboBox::from_label(": Y")
                    .selected_text(self.selected_y_column.as_deref().unwrap_or("Select Y Column"))
                    .show_ui(ui, |ui| {
                        for &name in CUT_COLUMN_NAMES {
                            if ui.selectable_label(self.selected_y_column.as_deref() == Some(name), name).clicked() {
                                self.selected_y_column = Some(name.to_string());
                            }
                        }
                    });
    
                ui.separator();
    
                // The "Save Cut" button is always visible, but only enabled when both X and Y columns are selected
                let save_button_enabled = self.selected_x_column.is_some() && self.selected_y_column.is_some();
                // Use ui.add_enabled to add the button in either enabled or disabled state
                if ui.add_enabled(save_button_enabled, egui::Button::new("Save Cut")).clicked() {
                    if let Err(e) = self.save_cut_to_json() {
                        eprintln!("Failed to save cut: {:?}", e);
                    }
                }

                ui.separator();
    
                if ui.button("Clear Cut").clicked() {
                    self.remove_all_vertices();
                    self.selected_x_column = None;
                    self.selected_y_column = None;
                }
            }
        });
    }

    pub fn draw(&mut self, plot_ui: &mut PlotUi) {
        self.handle_mouse_interactions(plot_ui);   // Handle mouse interactions
        self.draw_vertices_and_polygon(plot_ui);   // Draw vertices and polygon
    }

    // Save the current state of the polygon to a JSON file.
    fn save_cut_to_json(&self) -> Result<(), Box<dyn std::error::Error>> {

        let default_filename = if let (Some(y_col), Some(x_col)) = (&self.selected_y_column, &self.selected_x_column) {
            format!("{}_{}_cut.json", y_col, x_col)
        } else {
            "cut.json".to_string()
        };

        if let Some(file_path) = FileDialog::new()
            .set_file_name(&default_filename) 
            .add_filter("JSON Files", &["json"])  // Add a filter for json files
            .save_file() {

                let serialized = serde_json::to_string(self)?;
                let mut file = File::create(file_path)?;
                file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }

    // Load the polygon from a JSON file.
    fn load_cut_from_json(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_path) = FileDialog::new()
            .set_file_name("cut.json")  // Suggest a default file name for convenience
            .add_filter("JSON Files", &["json"])  // Filter for json files
            .pick_file() {

                let file = File::open(file_path)?;
                let reader = BufReader::new(file);
                let loaded_polygon: EditablePolygon = serde_json::from_reader(reader)?;
                *self = loaded_polygon;
        }
        Ok(())
    }

    fn handle_mouse_interactions(&mut self, plot_ui: &mut PlotUi) {
        let response = plot_ui.response();

        if response.clicked() {
            let pointer_pos = plot_ui.pointer_coordinate().unwrap();
            self.add_new_vertex([pointer_pos.x, pointer_pos.y]); // Add a new vertex on left-click
        }

        if response.secondary_clicked() {
            let pointer_pos = plot_ui.pointer_coordinate().unwrap();
            self.selected_vertex_index = self.get_closest_vertex_index([pointer_pos.x, pointer_pos.y]); // Select and remove on right-click
            self.remove_vertex();
        }

        if response.middle_clicked() {
            self.remove_all_vertices(); // Remove all vertices on middle-click
        }
    }

    fn add_new_vertex(&mut self, coordinates: [f64; 2]) {
        self.vertices.push(coordinates); // Add a new vertex to the list
    }

    fn remove_vertex(&mut self) {
        if let Some(index) = self.selected_vertex_index {
            self.vertices.remove(index); // Remove the selected vertex
            self.clear_selection(); // Clear the selection
        }
    }

    // Find the index of the vertex closest to the pointer's position
    fn get_closest_vertex_index(&self, pointer_pos: [f64; 2]) -> Option<usize> {
        let mut closest_vertex_index: Option<usize> = None;
        let mut closest_distance: f64 = 0.0;

        for (index, vertex) in self.vertices.iter().enumerate() {
            let distance = (vertex[0] - pointer_pos[0]).powi(2) + (vertex[1] - pointer_pos[1]).powi(2);
            if closest_vertex_index.is_none() || distance < closest_distance {
                closest_vertex_index = Some(index);
                closest_distance = distance;
            }
        }

        closest_vertex_index
    }

    fn clear_selection(&mut self) {
        self.selected_vertex_index = None; // Clear the selected vertex
    }

    pub fn remove_all_vertices(&mut self) {
        self.vertices.clear(); // Remove all vertices
        self.clear_selection(); // Clear the selection
    }

    fn draw_vertices_and_polygon(&mut self, plot_ui: &mut PlotUi) {
        if !self.vertices.is_empty() {
            let color = Color32::RED;
            let plot_points = PlotPoints::new(self.vertices.clone());
            let polygon_points = Polygon::new(plot_points).fill_color(Color32::TRANSPARENT).stroke(Stroke::new(4.0, color));
            plot_ui.polygon(polygon_points); // Draw the polygon

            let vertices = Points::new(self.vertices.clone()).radius(5.0).color(color);
            plot_ui.points(vertices); // Draw the vertices
        }
    }
}


