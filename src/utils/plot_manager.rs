use super::histogrammer::{Histogrammer, HistogramTypes};
use egui_plot::{Plot, Legend, Text, PlotPoint};
use eframe::egui::{self, Color32};

use crate::utils::cut::CutHandler;

pub struct PlotManager {
    pub histogrammer: Histogrammer,
    selected_histograms: Vec<String>,
    pub cutter: CutHandler,
}

impl PlotManager {

    pub fn new(histogrammer: Histogrammer, cutter: CutHandler) -> Self {

        Self {
            histogrammer,
            selected_histograms: Vec::new(),
            cutter,
        }
    }

    fn get_histogram_list(&self) -> Vec<String> {
        // Retrieves a sorted list of histogram names.
        let mut histogram_names: Vec<String> = self.histogrammer.histogram_list
            .keys()
            .cloned()
            .collect();
        histogram_names.sort();
        histogram_names
    }

    fn get_histogram_type(&self, name: &str) -> Option<&HistogramTypes> {
        self.histogrammer.histogram_list.get(name)
    }

    pub fn render_buttons(&mut self, ui: &mut egui::Ui) {

        ui.label("Histograms"); // Label for the histogram buttons.
        
        let keys: Vec<String> = self.get_histogram_list(); // Retrieve the list of histogram names.

        // Layout for the buttons: top down and justified at the top.
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
            for name in keys {
                // Create a button for each histogram name.
                let button: egui::Button<'_> = egui::Button::new(&name);
                let response: egui::Response = ui.add(button); // Add the button to the UI and get the response.

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

    pub fn render_selected_histograms(&mut self, ui: &mut egui::Ui) {
        // Display a message if no histograms are selected.
        if self.selected_histograms.is_empty() {
            ui.label("No histogram selected");
            return;
        }

        // Set up the plot for the combined histogram display.
        let plot = Plot::new("Combined Histogram")
            .legend(Legend::default())
            .clamp_grid(true)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_boxed_zoom(true)
            .allow_scroll(true);

        
        // Display the plot in the UI.
        plot.show(ui, |plot_ui| {

            // Define a set of colors for the histograms.
            let colors: [Color32; 5] = [
                Color32::LIGHT_BLUE, 
                Color32::LIGHT_RED, 
                Color32::LIGHT_GREEN, 
                Color32::LIGHT_YELLOW, 
                Color32::LIGHT_GRAY
            ];

            let plot_min_x = plot_ui.plot_bounds().min()[0];
            let plot_max_x = plot_ui.plot_bounds().max()[0];
            let plot_min_y = plot_ui.plot_bounds().min()[1];
            let plot_max_y = plot_ui.plot_bounds().max()[1];

            for (i, selected_name) in self.selected_histograms.iter().enumerate() {
                // Render the appropriate histogram type based on its type.
                match self.get_histogram_type(selected_name) {
                    Some(HistogramTypes::Hist1D(hist)) => {

                        // Render a 1D histogram as a step line.
                        let hist_color = colors[i % colors.len()];
                        // if let Some(step_line) = self.histogrammer.egui_histogram_step(selected_name, colors[i % colors.len()]) {
                        if let Some(step_line) = self.histogrammer.egui_histogram_step(selected_name, hist_color) {

                            plot_ui.line(step_line);

                            let stats_entries = hist.legend_entries(plot_min_x, plot_max_x);

                            for (_i, entry) in stats_entries.iter().enumerate() {
                                plot_ui.text(
                                    Text::new(PlotPoint::new(0, 0), " ") // Placeholder for positioning; adjust as needed
                                        .highlight(false)
                                        .color(hist_color)
                                        .name(entry)
                                );
                            }

                        }
                    }
                    Some(HistogramTypes::Hist2D(hist)) => {
                        
                        let hist_color = colors[i % colors.len()];

                        // Render a 2D histogram as a heatmap.
                        if let Some(bar_chart) = self.histogrammer.egui_heatmap(selected_name) {
                            plot_ui.bar_chart(bar_chart);

                            let stats_entries = hist.legend_entries(plot_min_x, plot_max_x, plot_min_y, plot_max_y);

                            for (_i, entry) in stats_entries.iter().enumerate() {
                                plot_ui.text(
                                    Text::new(PlotPoint::new(0, 0), " ") // Placeholder for positioning; adjust as needed
                                        .highlight(false)
                                        .color(hist_color)
                                        .name(entry)
                                );
                            }

                        }
                    }

                    None => {
                        // Optionally handle the case where the histogram is not found or its type is not supported.
                        // ui.label(format!("Histogram '{}' not found or type not supported.", selected_name));
                    }
                }
            }

            self.cutter.draw_active_cut(plot_ui);
            
        });
    }

    
}