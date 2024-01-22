use ndarray::Array1;
use ndhistogram::{ndhistogram, Histogram, VecHistogram, AxesTuple, axis::Uniform};
use std::collections::HashMap;
use eframe::egui::{Color32, Stroke};

use egui_plot::{Bar, Orientation, BarChart, Line, PlotPoints};
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
    pub min_value: f64, // Minimum histogram value
    pub max_value: f64, // Maximum histogram value
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

        // Initialize min and max values
        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;

        // Calculate min and max values based on histogram data
        for item in hist.iter() {
            let count = *item.value;
            min_value = min_value.min(count);
            max_value = max_value.max(count);
        }

        // Hist2D { name, x_range, x_bin_width, y_range, y_bin_width, hist }
        Hist2D { name, x_range, x_bin_width, y_range, y_bin_width, hist, min_value, max_value }
    }

    pub fn update_min_max_values(&mut self) {
        self.min_value = f64::INFINITY;
        self.max_value = f64::NEG_INFINITY;

        for item in self.hist.iter() {
            let count = *item.value;
            self.min_value = self.min_value.min(count);
            self.max_value = self.max_value.max(count);
        }
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
}

impl Histogrammer {

    // Creates a new instance of Histogrammer.
    pub fn new() -> Self {
        Self {
            histogram_list: HashMap::new(), 
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

        // Update min and max values after filling the histogram
        hist.update_min_max_values(); // Assuming this is a method in Hist2D

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

            let min = hist.min_value;
            let max = hist.max_value;

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
    
}

fn viridis_colormap(value: f64, min: f64, max: f64) -> Color32 {
    // Apply logarithmic normalization if required
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);

    // Key colors from the Viridis colormap
    let viridis_colors: [(f32, f32, f32); 32] = [
        (0.267003985, 0.004872566, 0.329415069),
        (0.277228998, 0.051716984, 0.37694991),
        (0.28247969, 0.097334964, 0.419510575),
        (0.282711276, 0.139317688, 0.456197068),
        (0.278092635, 0.179895883, 0.486377421),
        (0.269137787, 0.219429659, 0.50989087),
        (0.256733532, 0.257754383, 0.52718378),
        (0.242031461, 0.294643816, 0.539209024),
        (0.226243756, 0.329989329, 0.547162826),
        (0.210443168, 0.363856061, 0.552221276),
        (0.195412486, 0.396435844, 0.555350926),
        (0.181477325, 0.428017314, 0.557198854),
        (0.168574228, 0.458905237, 0.55806733),
        (0.156365949, 0.489384598, 0.557941172),
        (0.144535294, 0.519685615, 0.556527663),
        (0.133249552, 0.549958247, 0.553339219),
        (0.123833067, 0.580259243, 0.547771637),
        (0.119442112, 0.610546221, 0.53918201),
        (0.124881902, 0.640695014, 0.526954942),
        (0.144277738, 0.670499732, 0.510554716),
        (0.178281445, 0.699705646, 0.489567134),
        (0.224797439, 0.72801441, 0.463677887),
        (0.281243458, 0.755097766, 0.432683204),
        (0.345693489, 0.780604757, 0.396465689),
        (0.416705432, 0.80418531, 0.355029985),
        (0.493228829, 0.825506231, 0.308497657),
        (0.574270238, 0.844288831, 0.257257704),
        (0.658654029, 0.860389968, 0.202434461),
        (0.744780537, 0.873933018, 0.147547821),
        (0.830610047, 0.885437755, 0.10427358),
        (0.91400241, 0.895811264, 0.100134278),
        (0.993248149, 0.906154763, 0.143935944),
    ];

    // Interpolate between colors in the colormap
    let scaled_val = normalized * (viridis_colors.len() - 1) as f64;
    let index = scaled_val.floor() as usize;
    let fraction = scaled_val.fract() as f32;

    let color1 = viridis_colors[index];
    let color2 = viridis_colors[(index + 1).min(viridis_colors.len() - 1)];

    let red = (color1.0 + fraction * (color2.0 - color1.0)) * 255.0;
    let green = (color1.1 + fraction * (color2.1 - color1.1)) * 255.0;
    let blue = (color1.2 + fraction * (color2.2 - color1.2)) * 255.0;

    Color32::from_rgb(red as u8, green as u8, blue as u8)
}

fn column_to_array1(dataframe: &LazyFrame, column_name: &str) -> Result<Array1<f64>, PolarsError> {
    // Collect the DataFrame
    let df = dataframe
        .clone()
        .select([col(column_name)])
        .filter(col(column_name).neq(lit(-1e6))) // Filter out -1e6 values
        .collect()?;

    // Extract the column as a Series
    let series = df.column(column_name)?;

    // Convert the Series to ChunkedArray<f64>
    let chunked_array = series.f64()?;

    // Convert the ChunkedArray<f64> to an ndarray view
    let array_view = chunked_array.to_ndarray()?;

    // Convert the view to an owned Array1<f64>
    let array_owned = array_view.to_owned();

    Ok(array_owned)
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

    let array_view_x = chunked_array_x.to_ndarray()?;
    let array_view_y = chunked_array_y.to_ndarray()?;

    let array_owned_x = array_view_x.to_owned();
    let array_owned_y = array_view_y.to_owned();

    // Convert the Vecs into Array1<f64>
    Ok((array_owned_x, array_owned_y))
}
