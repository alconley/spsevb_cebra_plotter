use fnv::FnvHashMap;

// uses a hash map to store the histogram data (zero overhead for empty bins)
pub struct Histogram2D {
    pub bins: FnvHashMap<(usize, usize), u32>,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub x_bin_width: f64,
    pub y_bin_width: f64,
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
        }
    }

    // Add a value to the histogram
    pub fn add(&mut self, x_value: f64, y_value: f64) {
        if x_value >= self.x_range.0 && x_value < self.x_range.1 && y_value >= self.y_range.0 && y_value < self.y_range.1 {
            let x_index = ((x_value - self.x_range.0) / self.x_bin_width) as usize;
            let y_index = ((y_value - self.y_range.0) / self.y_bin_width) as usize;
            *self.bins.entry((x_index, y_index)).or_insert(0) += 1;
        }
    }

    // Additional methods to retrieve or manipulate histogram data can be added as needed
}