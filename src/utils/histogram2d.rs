use fnv::FnvHashMap;

// Define the BarData struct
pub struct BarData {
    pub x: f64,
    pub y: f64,
    pub bar_width: f64,
    pub height: f64,
    pub count: u32,
}

// uses a hash map to store the histogram data (zero overhead for empty bins)
pub struct Histogram2D {
    pub bins: FnvHashMap<(usize, usize), u32>,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub x_bin_width: f64,
    pub y_bin_width: f64,
    pub min_count: u32,
    pub max_count: u32,
}

impl Histogram2D {
    // Create a new 2D Histogram with specified ranges and number of bins for each axis
    pub fn new(x_bins: usize, x_range: (f64, f64), y_bins: usize, y_range: (f64, f64)) -> Self {
        Histogram2D {
            bins: FnvHashMap::default(),
            x_range,
            y_range,
            x_bin_width: (x_range.1 - x_range.0) / x_bins as f64,
            y_bin_width: (y_range.1 - y_range.0) / y_bins as f64,
            min_count: u32::MAX,
            max_count: u32::MIN,
        }
    }

    // Add a value to the histogram
    pub fn fill(&mut self, x_value: f64, y_value: f64) {
        if x_value >= self.x_range.0 && x_value < self.x_range.1 && y_value >= self.y_range.0 && y_value < self.y_range.1 {
            let x_index = ((x_value - self.x_range.0) / self.x_bin_width) as usize;
            let y_index = ((y_value - self.y_range.0) / self.y_bin_width) as usize;
            let count = self.bins.entry((x_index, y_index)).or_insert(0);
            *count += 1;

            // Update min and max counts
            if *count < self.min_count {
                self.min_count = *count;
            }
            if *count > self.max_count {
                self.max_count = *count;
            }
        }
    }

    // Method to generate data for egui heatmap
    pub fn generate_bar_data(&self) -> Vec<BarData> {
        let mut bars = Vec::new();

        for (&(x_index, y_index), &count) in &self.bins {
            if count == 0 {
                continue; // Skip empty bins
            }

            let x_bin_start = self.x_range.0 + x_index as f64 * self.x_bin_width;
            let x_bin_end = x_bin_start + self.x_bin_width;
            let y_bin_start = self.y_range.0 + y_index as f64 * self.y_bin_width;
            let y_bin_end = y_bin_start + self.y_bin_width;

            bars.push(BarData {
                x: (x_bin_start + x_bin_end) / 2.0,
                y: (y_bin_start + y_bin_end) / 2.0,
                bar_width: self.x_bin_width,
                height: self.y_bin_width,
                count,
            });
        }

        bars
    }

    // Additional methods to retrieve or manipulate histogram data can be added as needed
}