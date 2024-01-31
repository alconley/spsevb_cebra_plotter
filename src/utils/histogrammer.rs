use ndarray::Array1;
use std::collections::HashMap;
use eframe::egui::{Color32, Stroke};

use egui_plot::{Bar, Orientation, BarChart, Line, PlotPoints};
use polars::prelude::*;

use crate::utils::histogram1d::Histogram;
use crate::utils::histogram2d::Histogram2D;

pub enum HistogramTypes {
    Hist1D(Histogram),
    Hist2D(Histogram2D) 

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
        let hist: Histogram = Histogram::new(bins, range); // Create a new histogram.
        self.histogram_list.insert(name.to_string(), HistogramTypes::Hist1D(hist)); // Store it in the hashmap.
    }

    // Fills a 1D histogram with data.
    pub fn fill_hist1d(&mut self, name: &str, data: Array1<f64>) -> bool {
        let hist: &mut Histogram = match self.histogram_list.get_mut(name) {
            Some(HistogramTypes::Hist1D(hist)) => hist,
            _ => return false,  // Return false if the histogram doesn't exist.
        };

        data.iter().for_each(|&value| hist.fill(value)); // Fill the histogram with data.
        
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
            let line_points = hist.step_histogram_points();

            // Convert line_points to a Vec<[f64; 2]>
            let plot_points: PlotPoints = line_points.iter().map(|&(x, y)| [x, y]).collect();

            Some(Line::new(plot_points).color(color).name(name))

        } else {
            None
        }
    }
    
    // Adds a new 2D histogram to the histogram list.
    pub fn add_hist2d(&mut self, name: &str, x_bins: usize, x_range: (f64, f64), y_bins: usize, y_range: (f64, f64)) {
        let hist: Histogram2D = Histogram2D::new(x_bins, x_range, y_bins, y_range); // Create a new 2D histogram.
        self.histogram_list.insert(name.to_string(), HistogramTypes::Hist2D(hist)); // Store it in the hashmap.
    }

    // Fills a 2D histogram with x and y data.
    pub fn fill_hist2d(&mut self, name: &str, x_data: Array1<f64>, y_data: Array1<f64>) -> bool {
        let hist: &mut Histogram2D = match self.histogram_list.get_mut(name) {
            Some(HistogramTypes::Hist2D(hist)) => hist,
            _ => return false, // Return false if the histogram doesn't exist.
        };

        if x_data.len() != y_data.len() {
            eprintln!("Error: x_data and y_data lengths do not match.");
            return false; // Ensure that the lengths of x and y data arrays are equal.
        }

        for (&x, &y) in x_data.iter().zip(y_data.iter()) {
            hist.fill(x, y); // Fill the histogram with the (x, y) pairs.
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
            let bars_data = hist.generate_bar_data();           
            let mut bars = Vec::new();

            let min: u32 = hist.min_count;
            let max: u32 = hist.max_count;
            for bar_data in bars_data {

                let color: Color32 = viridis_colormap(bar_data.count, min, max); // Determine color based on the count, using a colormap.
                
                let bar = Bar {
                    orientation: Orientation::Vertical,
                    argument: bar_data.x,
                    value: bar_data.height,
                    bar_width: bar_data.bar_width,
                    fill: color,
                    stroke: Stroke::new(1.0, color),
                    name: format!("x = {}\ny = {}\n{}", bar_data.x, bar_data.y, bar_data.count),
                    base_offset: Some(bar_data.y - bar_data.height / 2.0),
                };
                bars.push(bar);

            }
    
            // Return a BarChart object if the histogram exists, otherwise return None.
            Some(BarChart::new(bars).name(name))
        } else {
            None
        }
    }
        
}

fn viridis_colormap(value: u32, min: u32, max: u32) -> Color32 {
    // Handle case where min == max to avoid division by zero
    let normalized: f64 = if max > min {
        (value as f64 - min as f64) / (max as f64 - min as f64)
    } else {
        0.0
    }.clamp(0.0, 1.0);

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
    let scaled_val: f64 = normalized * (viridis_colors.len() - 1) as f64;
    let index: usize = scaled_val.floor() as usize;
    let fraction: f32 = scaled_val.fract() as f32;

    let color1: (f32, f32, f32) = viridis_colors[index];
    let color2: (f32, f32, f32) = viridis_colors[(index + 1).min(viridis_colors.len() - 1)];

    let red: f32 = (color1.0 + fraction * (color2.0 - color1.0)) * 255.0;
    let green: f32 = (color1.1 + fraction * (color2.1 - color1.1)) * 255.0;
    let blue: f32 = (color1.2 + fraction * (color2.2 - color1.2)) * 255.0;

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
    let series: &Series = df.column(column_name)?;

    // Convert the Series to ChunkedArray<f64>
    let chunked_array: &ChunkedArray<Float64Type> = series.f64()?;

    // Convert the ChunkedArray<f64> to an ndarray view
    let array_view = chunked_array.to_ndarray()?;

    // Convert the view to an owned Array1<f64>
    let array_owned = array_view.to_owned();

    Ok(array_owned)
}

fn columns_to_array1(dataframe: &LazyFrame, x_column_name: &str, y_column_name: &str) -> Result<(Array1<f64>, Array1<f64>), PolarsError> {
    // Select and filter the column, then collect into a DataFrame
    let df: DataFrame = dataframe.clone()
        .select([col(x_column_name), col(y_column_name)])
        .filter(col(x_column_name).neq(lit(-1e6)))
        .filter(col(y_column_name).neq(lit(-1e6)))
        .collect()?;

    let series_x: &Series = df.column(x_column_name)?;
    let series_y: &Series = df.column(y_column_name)?;

    // Try to convert the Series into a ChunkedArray of f64
    let chunked_array_x: &ChunkedArray<Float64Type> = series_x.f64()?;
    let chunked_array_y: &ChunkedArray<Float64Type> = series_y.f64()?;

    let array_view_x = chunked_array_x.to_ndarray()?;
    let array_view_y = chunked_array_y.to_ndarray()?;

    let array_owned_x = array_view_x.to_owned();
    let array_owned_y = array_view_y.to_owned();

    // Convert the Vecs into Array1<f64>
    Ok((array_owned_x, array_owned_y))
}
