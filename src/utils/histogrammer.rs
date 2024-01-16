use ndarray::{Array1};
use ndhistogram::{ndhistogram, Histogram, VecHistogram, AxesTuple, axis::{Uniform}};
use std::collections::HashMap;
use eframe::egui::{Color32, Stroke};
use egui_plot::{Plot, Bar, Orientation, BarChart, Legend, Line, PlotPoints};
use polars::prelude::*;


/// Represents a one-dimensional histogram.
pub struct Hist1D {
    pub name: String,
    pub range: (f64, f64),
    pub bin_width: f64,
    pub hist: VecHistogram<AxesTuple<(Uniform<f64>,)>, f64>,
}

impl Hist1D {
    /// Creates a new `Hist1D` with the specified parameters.
    ///
    /// # Arguments
    /// * `name` - A name for the histogram.
    /// * `bins` - The number of bins.
    /// * `range` - The range (min, max) of the histogram.
    ///
    /// # Returns
    /// A new `Hist1D` instance.
    pub fn new(name: String, bins: usize, range: (f64, f64)) -> Hist1D {
        let bin_width = (range.1 - range.0) / bins as f64;
        let hist = ndhistogram!(Uniform::<f64>::new(bins, range.0, range.1));

        Hist1D { name, range, bin_width, hist }
    }

    // /// Get the bin number for a given x position.
    // pub fn get_bin(&self, x: f64) -> Option<usize> {
    //     // Retrieve the first (and only) axis from the histogram
    //     let axis = &self.hist.axes();

    //     // Use the `index` method to find the bin number for the given x
    //     axis.index(&x)
    // }

    // Additional methods for Hist1D could be implemented here
    // like getting the mean, std, counts
}

/// Represents a two-dimensional histogram.
pub struct Hist2D {
    pub name: String,
    pub x_range: (f64, f64),
    pub x_bin_width: f64,
    pub y_range: (f64, f64),
    pub y_bin_width: f64,
    pub hist: VecHistogram<AxesTuple<(Uniform<f64>, Uniform<f64>)>, f64>,
}

impl Hist2D {
    /// Creates a new `Hist2D` with the specified parameters.
    ///
    /// # Arguments
    /// * `name` - A name for the histogram.
    /// * `bins_x` - The number of bins for the X axis.
    /// * `bins_y` - The number of bins for the Y axis.
    /// * `x_range` - The range (min, max) for the X axis.
    /// * `y_range` - The range (min, max) for the Y axis.
    ///
    /// # Returns
    /// A new `Hist2D` instance.
    pub fn new(name: String, x_bins: usize, x_range: (f64, f64), y_bins: usize, y_range: (f64, f64)) -> Hist2D {
        let x_bin_width = (x_range.1 - x_range.0) / x_bins as f64;
        let y_bin_width = (y_range.1 - y_range.0) / y_bins as f64;

        let hist = ndhistogram!(
            Uniform::new(x_bins, x_range.0, x_range.1),
            Uniform::new(y_bins, y_range.0, y_range.1)
        );

        Hist2D { name, x_range, x_bin_width, y_range, y_bin_width, hist }
    }

    // Additional methods for Hist2D could be implemented here
}

pub enum HistogramTypes {
    Hist1D(Hist1D),
    Hist2D(Hist2D) 
}

#[derive(Default)]
pub struct Histogrammer {
    pub histogram_list: HashMap<String, HistogramTypes>,
    // pub selected_histogram: Option<String>,
    pub selected_histograms: Vec<String>,

}

impl Histogrammer {

    // Creates a new instance of Histogrammer.
    pub fn new() -> Self {
        Self {
            histogram_list: HashMap::new(),  // HashMap to store histograms by name.
            // selected_histograms: A vector to store selected histograms (currently unused).
            selected_histograms: Vec::new(),
        }
    }

    // Adds a new 1D histogram to the histogram list.
    pub fn add_hist1d(&mut self, name: &str, bins: usize, range: (f64, f64)) {
        let hist = Hist1D::new(name.to_string(), bins, range); // Create a new histogram.
        self.histogram_list.insert(name.to_string(), HistogramTypes::Hist1D(hist)); // Store it in the hashmap.
    }

    // Fills a 1D histogram with data.
    pub fn fill_hist1d(&mut self, name: &str, data: Array1<f64>) -> bool {
        let hist = match self.histogram_list.get_mut(name) {
            Some(HistogramTypes::Hist1D(hist)) => hist,
            _ => return false,  // Return false if the histogram doesn't exist.
        };

        data.iter().for_each(|&value| hist.hist.fill(&value)); // Fill the histogram with data.

        true
    }

    // Fills a 1D histogram with data from a Polars LazyFrame.
    pub fn fill_hist1d_from_polars(&mut self, name: &str, lf: &LazyFrame, column_name: &str) {
        match column_to_array1(lf, column_name) {
            Ok(data) => {
                if !self.fill_hist1d(name, data) {  // Fill the histogram with the data.
                    eprintln!("Failed to fill histogram '{}' with data from column '{}'.", name, column_name);
                }
            }
            Err(e) => {
                eprintln!("Error extracting data from column '{}': {:?}", column_name, e);
            }
        }
    }

    // Adds and fills a 1D histogram with data from a Polars LazyFrame.
    pub fn add_fill_hist1d_from_polars(&mut self, name: &str, lf: &LazyFrame, column_name: &str, bins: usize, range: (f64, f64)) {
        self.add_hist1d(name, bins, range);  // Add the histogram.
        self.fill_hist1d_from_polars(name, lf, column_name);  // Fill it with data.
    }

    // Generates a histogram using the bar chart from the `egui` library.
    pub fn egui_histogram_step(&self, name: &str, color: Color32) -> Option<Line> {
        if let Some(HistogramTypes::Hist1D(hist)) = self.histogram_list.get(name) {
            let mut line_points = Vec::new();

            let bin_width = hist.bin_width; // Width of each bin in the histogram.

            for item in hist.hist.iter() {
                let start = item.bin.start().unwrap_or(f64::NEG_INFINITY); // Start of the bin.
                let end = item.bin.end().unwrap_or(f64::INFINITY); // End of the bin.
    
                // Skip bins with infinite bounds.
                if start.is_infinite() || end.is_infinite() {
                    continue;
                }

                // Add points for the line at the start and end of each bar
                line_points.push((start, *item.value));
                line_points.push((end, *item.value));
        
            }

            // Convert line_points to a Vec<[f64; 2]>
            let plot_points: PlotPoints = line_points.iter().map(|&(x, y)| [x, y]).collect();

            Some(Line::new(plot_points).color(color).name(name))

        } else {
            None
        }
    }
    
    // Adds a new 2D histogram to the histogram list.
    pub fn add_hist2d(&mut self, name: &str, x_bins: usize, x_range: (f64, f64), y_bins: usize, y_range: (f64, f64)) {
        let hist = Hist2D::new(name.to_string(), x_bins, x_range, y_bins, y_range); // Create a new 2D histogram.
        self.histogram_list.insert(name.to_string(), HistogramTypes::Hist2D(hist)); // Store it in the hashmap.
    }

    // Fills a 2D histogram with x and y data.
    pub fn fill_hist2d(&mut self, name: &str, x_data: Array1<f64>, y_data: Array1<f64>) -> bool {
        let hist = match self.histogram_list.get_mut(name) {
            Some(HistogramTypes::Hist2D(hist)) => hist,
            _ => return false, // Return false if the histogram doesn't exist.
        };

        if x_data.len() != y_data.len() {
            eprintln!("Error: x_data and y_data lengths do not match.");
            return false; // Ensure that the lengths of x and y data arrays are equal.
        }

        for (&x, &y) in x_data.iter().zip(y_data.iter()) {
            hist.hist.fill(&(x, y)); // Fill the histogram with the (x, y) pairs.
        }

        true
    }
    
    // Fills a 2D histogram with data from Polars LazyFrame columns.
    pub fn fill_hist2d_from_polars(&mut self, name: &str, lf: &LazyFrame, x_column_name: &str, y_column_name: &str) {
        match columns_to_array1(lf, x_column_name, y_column_name) {
            Ok((x_data, y_data)) => {
                if !self.fill_hist2d(name, x_data, y_data) { // Fill the histogram with the extracted data.
                    eprintln!("Failed to fill histogram '{}' with data from columns '{}' and '{}'.", name, x_column_name, y_column_name);
                }
            }
            Err(e) => {
                eprintln!("Error extracting data from columns '{}' and '{}': {:?}", x_column_name, y_column_name, e);
            }
        }
    }

    // Adds and fills a 2D histogram with data from Polars LazyFrame columns.
    pub fn add_fill_hist2d_from_polars(&mut self, name: &str, lf: &LazyFrame, x_column_name: &str, x_bins: usize, x_range: (f64, f64), y_column_name: &str, y_bins: usize, y_range: (f64, f64)) {
        self.add_hist2d(name, x_bins, x_range, y_bins, y_range); // Add the histogram.
        self.fill_hist2d_from_polars(name, lf, x_column_name, y_column_name); // Fill it with data.
    }

    // Generates a heatmap using the `egui` library based on a 2D histogram.
    pub fn egui_heatmap(&self, name: &str) -> Option<BarChart> {
        if let Some(HistogramTypes::Hist2D(hist)) = self.histogram_list.get(name) {
            let mut bars = Vec::new();

            // Calculate the minimum and maximum values in the histogram for color mapping.
            let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
            for item in hist.hist.iter() {
                let count = *item.value;
                min = min.min(count);
                max = max.max(count);
            }

            for item in hist.hist.iter() {
                let (x_bin, y_bin) = item.bin;
                let count = *item.value;

                // Skip bins with a count of 0 to improve performance.
                if count == 0.0 {
                    continue;
                }

                let x_bin_start = x_bin.start().unwrap_or(f64::NEG_INFINITY); // Start of the x bin.
                let x_bin_end = x_bin.end().unwrap_or(f64::INFINITY); // End of the x bin.
    
                let y_bin_start = y_bin.start().unwrap_or(f64::NEG_INFINITY); // Start of the y bin.
                let y_bin_end = y_bin.end().unwrap_or(f64::INFINITY); // End of the y bin.
    
                // Skip bins with infinite bounds to avoid rendering issues.
                if x_bin_start.is_infinite() || x_bin_end.is_infinite() || y_bin_start.is_infinite() || y_bin_end.is_infinite() {
                    continue;
                }
    
                let x = (x_bin_start + x_bin_end) / 2.0; // Midpoint of the x bin.
                let y = (y_bin_start + y_bin_end) / 2.0; // Midpoint of the y bin.
                let bar_width = x_bin_end - x_bin_start; // Width of the x bin.
                let height = y_bin_end - y_bin_start; // Height of the y bin.
                
                let color = viridis_colormap(count, min, max); // Determine color based on the count, using a colormap.
    
                // Create a bar to represent the 2D histogram data.
                let bar = Bar {
                    orientation: Orientation::Vertical,
                    argument: x, // X-coordinate of the bar.
                    value: height, // Height of the bar.
                    bar_width, // Width of the bar.
                    fill: color, // Color of the bar.
                    stroke: Stroke::new(1.0, color), // Border color and width of the bar.
                    name: format!("x = {}\ny = {}\n{}", x, y, count), // Label for the bar.
                    base_offset: Some(y_bin_start), // Offset from the base (Y-coordinate of the start of the bin).
                };
                bars.push(bar); // Add the bar to the vector.
            }
    
            // Return a BarChart object if the histogram exists, otherwise return None.
            Some(BarChart::new(bars).name(name))
        } else {
            None
        }
    }
    
    fn get_histogram_list(&self) -> Vec<String> {
        // Retrieves a sorted list of histogram names.
        let mut histogram_names: Vec<String> = self.histogram_list
            .keys()
            .cloned()
            .collect();
        histogram_names.sort();
        histogram_names
    }

    fn get_histogram_type(&self, name: &str) -> Option<&HistogramTypes> {
        self.histogram_list.get(name)
    }

   // Renders buttons for selecting histograms on the UI.
   pub fn render_buttons(&mut self, ui: &mut egui::Ui) {

        ui.label("Histograms"); // Label for the histogram buttons.
        
        let keys = self.get_histogram_list(); // Retrieve the list of histogram names.

        // Layout for the buttons: top down and justified at the top.
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
            for name in keys {
                // Create a button for each histogram name.
                let button = egui::Button::new(&name);
                let response = ui.add(button); // Add the button to the UI and get the response.

                // If the button is clicked, clear the current selection and select this histogram.
                if response.clicked() {
                    self.selected_histograms.clear();
                    self.selected_histograms.push(name.clone());
                }

                // If the button is right-clicked, add this histogram to the selection without clearing existing selections.
                if response.secondary_clicked() {
                    if !self.selected_histograms.contains(&name) {
                        self.selected_histograms.push(name.clone());
                    }
                }
            }
        });
    }

    // Renders the selected histograms on the UI.
    pub fn render_selected_histograms(&self, ui: &mut egui::Ui) {
        // Display a message if no histograms are selected.
        if self.selected_histograms.is_empty() {
            ui.label("No histogram selected");
            return;
        }

        // Set up the plot for the combined histogram display.
        let plot = Plot::new("Combined Histogram")
            .legend(Legend::default())
            .clamp_grid(true)
            .allow_drag(false);

        // Display the plot in the UI.
        plot.show(ui, |plot_ui| {
            // Define a set of colors for the histograms.
            let colors = [
                Color32::LIGHT_BLUE, 
                Color32::LIGHT_RED, 
                Color32::LIGHT_GREEN, 
                Color32::LIGHT_YELLOW, 
                Color32::LIGHT_GRAY
            ];
                
            for (i, selected_name) in self.selected_histograms.iter().enumerate() {
                // Render the appropriate histogram type based on its type.
                match self.get_histogram_type(selected_name) {
                    Some(HistogramTypes::Hist1D(_)) => {
                        // Render a 1D histogram as a step line.
                        if let Some(step_line) = self.egui_histogram_step(selected_name, colors[i % colors.len()]) {
                            plot_ui.line(step_line);
                        }
                    }
                    Some(HistogramTypes::Hist2D(_)) => {
                        // Render a 2D histogram as a heatmap.
                        if let Some(bar_chart) = self.egui_heatmap(selected_name) {
                            plot_ui.bar_chart(bar_chart);
                        }
                    }
                    None => {
                        // Optionally handle the case where the histogram is not found or its type is not supported.
                        // ui.label(format!("Histogram '{}' not found or type not supported.", selected_name));
                    }
                }
            }
        });
    }

}

// Function to create a color from the Viridis colormap
fn viridis_colormap(value: f64, min: f64, max: f64) -> Color32 {

    // Apply logarithmic normalization if required
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);

    // Key colors from the Viridis colormap
    let viridis_colors: [(f32, f32, f32); 5] = [
        (0.267004, 0.004874, 0.329415), // dark purple
        (0.229739, 0.322361, 0.545706), // blue
        (0.127568, 0.566949, 0.550556), // cyan
        (0.369214, 0.788888, 0.382914), // yellow-green
        (0.993248, 0.906157, 0.143936), // yellow
    ];

    // Interpolate between colors in the colormap
    let scaled_val = normalized * (viridis_colors.len() - 1) as f64;
    let index = scaled_val.floor() as usize;
    let fraction = scaled_val.fract() as f32;

    let color1 = viridis_colors[index];
    let color2 = viridis_colors[index.min(viridis_colors.len() - 2)];

    let red = (color1.0 + fraction * (color2.0 - color1.0)) * 255.0;
    let green = (color1.1 + fraction * (color2.1 - color1.1)) * 255.0;
    let blue = (color1.2 + fraction * (color2.2 - color1.2)) * 255.0;

    Color32::from_rgb(red as u8, green as u8, blue as u8)
}

fn column_to_array1(dataframe: &LazyFrame, column_name: &str) -> Result<Array1<f64>, PolarsError> {
    // Select and filter the column, then collect into a DataFrame
    let df = dataframe.clone()
        .select([col(column_name)])
        .filter(col(column_name).neq(lit(-1e6))) // our null values are -1e6, could be removed
        .collect()?;

    // Extract the column as a Series
    let series = df.column(column_name)?;

    // Try to convert the Series into a ChunkedArray of f64
    let chunked_array = series.f64()?;

    // Convert the ChunkedArray into a Vec<f64>, filtering out None values
    let vec: Vec<f64> = chunked_array.into_iter().filter_map(|opt| opt).collect();

    // Convert the Vec<f64> into Array1<f64>
    Ok(Array1::from(vec))
}

fn columns_to_array1(dataframe: &LazyFrame, x_column_name: &str, y_column_name: &str) -> Result<(Array1<f64>, Array1<f64>), PolarsError> {
    // Select and filter the column, then collect into a DataFrame
    let df = dataframe.clone()
        .select([col(x_column_name), col(y_column_name)])
        .filter(col(x_column_name).neq(lit(-1e6)))
        .filter(col(y_column_name).neq(lit(-1e6)))
        .collect()?;

    let series_x = df.column(x_column_name)?;
    let series_y = df.column(y_column_name)?;

    // Try to convert the Series into a ChunkedArray of f64
    let chunked_array_x = series_x.f64()?;
    let chunked_array_y = series_y.f64()?;

    // Convert the ChunkedArray into a Vec<f64>, filtering out None values
    let vec_x: Vec<f64> = chunked_array_x.into_iter().filter_map(|opt| opt).collect();
    let vec_y: Vec<f64> = chunked_array_y.into_iter().filter_map(|opt| opt).collect();

    // Convert the Vecs into Array1<f64>
    Ok((Array1::from(vec_x), Array1::from(vec_y)))
}
