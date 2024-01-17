use super::histogrammer::{Histogrammer, HistogramTypes};
use super::cut::EditablePolygon;
use egui_plot::{Plot, Legend};
use eframe::egui::{Color32};

#[derive(Default)]
pub struct PlotManager {
    // Fields necessary for plotting
    // For example, you might need a reference to the histogram data and the cut data
    pub histogrammer: Histogrammer,
    selected_histograms: Vec<String>,
    pub cut: EditablePolygon,
    // Other fields as necessary
}

impl PlotManager {
    // Initialize the new struct
    pub fn new(histogrammer: Histogrammer, cut: EditablePolygon) -> Self {
        Self {
            histogrammer,
            cut,
            selected_histograms: Vec::new(),
            // Initialize other fields
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
    pub fn render_selected_histograms(&mut self, ui: &mut egui::Ui, draw_cut: bool) {
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
                        if let Some(step_line) = self.histogrammer.egui_histogram_step(selected_name, colors[i % colors.len()]) {
                            plot_ui.line(step_line);
                        }
                    }
                    Some(HistogramTypes::Hist2D(_)) => {
                        // Render a 2D histogram as a heatmap.
                        if let Some(bar_chart) = self.histogrammer.egui_heatmap(selected_name) {
                            plot_ui.bar_chart(bar_chart);
                        }
                    }
                    None => {
                        // Optionally handle the case where the histogram is not found or its type is not supported.
                        // ui.label(format!("Histogram '{}' not found or type not supported.", selected_name));
                    }
                }
            }

            if draw_cut {
                self.cut.draw(plot_ui);
            }
        });
    }

    
}