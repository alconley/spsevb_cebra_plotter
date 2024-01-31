use egui_plot::{Points, PlotPoints, PlotUi};
use egui_plot::Polygon as EguiPolygon;

use eframe::egui::{Color32, Stroke};

use std::fs::File;
use std::io::{BufReader, Write};

use serde::{Serialize, Deserialize};
use serde_json;

use rfd::FileDialog;

use geo::{Point, Polygon, LineString, algorithm::contains::Contains};

use polars::prelude::*;


#[derive(Serialize, Deserialize, Default)]
pub struct EditableEguiPolygon {
    pub vertices: Vec<[f64; 2]>,        // List of vertex coordinates
    selected_vertex_index: Option<usize>,  // Index of the selected vertex (if any)
    pub selected_x_column: Option<String>,
    pub selected_y_column: Option<String>,
}

impl EditableEguiPolygon {
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

    pub fn draw(&mut self, plot_ui: &mut PlotUi) {
        self.handle_mouse_interactions(plot_ui);   // Handle mouse interactions
        self.draw_vertices_and_polygon(plot_ui);   // Draw vertices and polygon
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

    fn remove_all_vertices(&mut self) {
        self.vertices.clear(); // Remove all vertices
        self.clear_selection(); // Clear the selection
    }

    fn draw_vertices_and_polygon(&mut self, plot_ui: &mut PlotUi) {
        if !self.vertices.is_empty() {
            let color = Color32::RED;
            let plot_points = PlotPoints::new(self.vertices.clone());
            let polygon_points = EguiPolygon::new(plot_points).fill_color(Color32::TRANSPARENT).stroke(Stroke::new(4.0, color));
            plot_ui.polygon(polygon_points); // Draw the polygon

            let vertices = Points::new(self.vertices.clone()).radius(5.0).color(color);
            plot_ui.points(vertices); // Draw the vertices
        }
    }

    pub fn save_cut_to_json(&self) -> Result<(), Box<dyn std::error::Error>> {

        // Create a default file name based on the selected columns
        let default_name = match (&self.selected_x_column, &self.selected_y_column) {
                (Some(x), Some(y)) => format!("{}_{}_cut.json", y, x),
                _ => "cut.json".to_string(),
            };
    
        if let Some(file_path) = FileDialog::new()
            .set_file_name(default_name) 
            .add_filter("JSON Files", &["json"])  // Add a filter for json files
            .save_file() {
    
                let serialized = serde_json::to_string(self)?;
                let mut file = File::create(file_path)?;
                file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }

    pub fn load_cut_from_json(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        if let Some(file_path) = FileDialog::new()
            .set_file_name("cut.json")  // Suggest a default file name for convenience
            .add_filter("JSON Files", &["json"])  // Filter for json files
            .pick_file() {

                let file = File::open(file_path)?;
                let reader = BufReader::new(file);
                let loaded_polygon: EditableEguiPolygon = serde_json::from_reader(reader)?;
                *self = loaded_polygon;
        }
        Ok(())
    }

    fn to_geo_polygon(&self) -> Polygon<f64> {
        let exterior_coords: Vec<_> = self.vertices.iter()
            .map(|&[x, y]| (x, y))
            .collect();
        let exterior_line_string = LineString::from(exterior_coords);
        Polygon::new(exterior_line_string, vec![])
    }

    pub fn filter_dataframe(&self, dataframe: &LazyFrame, x_column_name: &str, y_column_name: &str) -> Result<LazyFrame, polars::error::PolarsError> {

        let df = dataframe.clone()
            // .select([col(x_column_name), col(y_column_name)])
            .filter(col(x_column_name).neq(lit(-1e6)))
            .filter(col(y_column_name).neq(lit(-1e6)))
            .collect()?;

        let x_col = df.column(x_column_name)?;
        let y_col = df.column(y_column_name)?;

        let polygon = self.to_geo_polygon();

        let mask = x_col.f64()?
            .into_iter()
            .zip(y_col.f64()?)
            .map(|(x, y)| {
                match (x, y) {
                    (Some(x), Some(y)) => {
                        let point = Point::new(x, y);
                        polygon.contains(&point)
                    },
                    _ => false,
                }
            })
            .collect::<BooleanChunked>();

        let filtered_df = df.filter(&mask)?.lazy();

        Ok(filtered_df)
    }

    /*
    pub fn filter_dataframe_test(&self, dataframe: LazyFrame) -> Result<DataFrame, polars::error::PolarsError> {

        // Takes a polars dataframe
        // creates a mask based on the polygon and columns
        // filters dataframe with the mask
        // Saves dataframe to a new file
        // The essentially destroys the lazy operation and should not be used on a large amount of data

        let x_column_name = self.selected_x_column.clone()
            .ok_or_else(|| PolarsError::ComputeError("X column name must be something".into()))?;
        let y_column_name = self.selected_y_column.clone()
            .ok_or_else(|| PolarsError::ComputeError("Y column name must be something".into()))?;

        let df = dataframe
            .filter(col(&x_column_name).neq(lit(-1e6)))
            .filter(col(&y_column_name).neq(lit(-1e6)))
            .collect()?;

        let x_col = df.column(&x_column_name)?;
        let y_col = df.column(&y_column_name)?;

        let polygon = self.to_geo_polygon();

        let mask = x_col.f64()?
            .into_iter()
            .zip(y_col.f64()?)
            .map(|(x, y)| {
                match (x, y) {
                    (Some(x), Some(y)) => {
                        let point = Point::new(x, y);
                        polygon.contains(&point)
                    },
                    _ => false,
                }
            })
            .collect::<BooleanChunked>();

        let filtered_df = df.filter(&mask)?;

        Ok(filtered_df)
    }
*/

}




