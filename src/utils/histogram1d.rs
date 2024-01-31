
pub struct Histogram {
    pub bins: Vec<u32>,
    pub range: (f64, f64),
    pub bin_width: f64,
}

impl Histogram {
    // Create a new Histogram with specified min, max, and number of bins
    pub fn new(number_of_bins: usize, range: (f64, f64)) -> Self {
        Histogram {
            bins: vec![0; number_of_bins],
            range : range,
            bin_width: (range.1 - range.0) / number_of_bins as f64,
        }
    }

    // Add a value to the histogram
    pub fn add(&mut self, value: f64) {
        if value >= self.range.0 && value < self.range.1 {
            let index = ((value - self.range.0) / self.bin_width) as usize;
            if index < self.bins.len() {
                self.bins[index] += 1;
            }
        }
    }

    // Get the bin number for a given x position.
    pub fn get_bin(&self, x: f64) -> Option<usize> {
        if x < self.range.0 || x > self.range.1 {
            return None;
        }
        
        let bin_index: usize = (((x - self.range.0)) / self.bin_width).floor() as usize;
        
        Some(bin_index)
    }

    // Method to calculate the sum of counts in a range of bins
    fn integral_in_range_bin(&self, start_bin: usize, end_bin: usize) -> u32 {
        let mut sum = 0;

        for bin in start_bin..=end_bin {
            if bin < self.bins.len() {
                sum += self.bins[bin];
            } else {
                break; // Exit the loop if bin index is out of range
            }
        }

        sum
    }

    // Method to calculate the sum of counts in a range based on x values
    fn integral_in_range_x(&self, start_x: f64, end_x: f64) -> u32 {
        let start_bin = self.get_bin(start_x).unwrap_or(0);
        let end_bin = self.get_bin(end_x).unwrap_or(self.bins.len() - 1);

        self.integral_in_range_bin(start_bin, end_bin)
    }

    // Method to calculate the weighted mean of counts in a range based on x values (returns x as bin center)
    fn mean_in_range_x(&self, start_x: f64, end_x: f64) -> f64 {
        let start_bin = self.get_bin(start_x).unwrap_or(0);
        let end_bin = self.get_bin(end_x).unwrap_or(self.bins.len() - 1);

        if start_bin > end_bin {
            return 0.0; // Return zero if the range is invalid
        }

        let mut sum_product = 0.0;
        let mut total_count = 0;

        for bin in start_bin..=end_bin {
            if bin < self.bins.len() {
                let bin_center = self.range.0 + (bin as f64 + 0.5) * self.bin_width;
                sum_product += self.bins[bin] as f64 * bin_center;
                total_count += self.bins[bin];
            } else {
                break;
            }
        }

        if total_count == 0 {
            0.0
        } else {
            sum_product / total_count as f64
        }
    }

    // Method to calculate the standard deviation of counts in a range based on x values. This is population stats, not sample stats. not sure what is more ideal for this application
    fn stdev_in_range_x(&self, start_x: f64, end_x: f64) -> f64 {
        let start_bin = self.get_bin(start_x).unwrap_or(0);
        let end_bin = self.get_bin(end_x).unwrap_or(self.bins.len() - 1);

        if start_bin > end_bin {
            return 0.0; // Return zero if the range is invalid
        }

        // Calculate mean first
        let mean = self.mean_in_range_x(start_x, end_x);
        let mut sum_squared_diff = 0.0;
        let mut total_count = 0;

        for bin in start_bin..=end_bin {
            if bin < self.bins.len() {
                let bin_center = self.range.0 + (bin as f64 + 0.5) * self.bin_width;
                let diff = bin_center - mean;
                sum_squared_diff += self.bins[bin] as f64 * diff * diff;
                total_count += self.bins[bin];
            } else {
                break;
            }
        }

        if total_count == 0 {
            0.0
        } else {
            (sum_squared_diff / total_count as f64).sqrt()
        }
    }
    
    pub fn step_histogram_points(&self) -> Vec<(f64, f64)> {
        let mut line_points: Vec<(f64, f64)> = Vec::new();

        for (index, &count) in self.bins.iter().enumerate() {
            let start = self.range.0 + index as f64 * self.bin_width; // Start of the bin
            let end = start + self.bin_width; // End of the bin

            // Add points for the line at the start and end of each bar
            line_points.push((start, count as f64));
            line_points.push((end, count as f64));
        }

        line_points
    }

    // Method to calculate integral, mean, and standard deviation in a range based on x values
    pub fn stats(&self, start_x: f64, end_x: f64) -> (u32, f64, f64) {
        let integral = self.integral_in_range_x(start_x, end_x);
        let mean = self.mean_in_range_x(start_x, end_x);
        let stdev = self.stdev_in_range_x(start_x, end_x);

        (integral, mean, stdev)
    }
    
}
