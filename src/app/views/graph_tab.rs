use eframe::egui;

use crate::app::NitroSenseApp;
use crate::graph::show_graph;
use crate::ui::theme::{card_surface_color, dim_text_color, inner_separator_color, panel_frame};

impl NitroSenseApp {
    pub(in crate::app::views) fn show_graph_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Temperature");
        ui.add_space(8.0);
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                graph_toggle_chip(ui, "CPU Temp", &mut self.graph_visibility.cpu_temp);
                graph_toggle_chip(ui, "GPU Temp", &mut self.graph_visibility.gpu_temp);
            });
            ui.add_space(8.0);
            show_graph(ui, &self.graph_history, &self.graph_visibility);
        });
    }
}

fn graph_toggle_chip(ui: &mut egui::Ui, label: &str, enabled: &mut bool) {
    let (fill, stroke, text_color) = if *enabled {
        (
            egui::Color32::from_rgb(45, 49, 55),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 96, 104)),
            egui::Color32::WHITE,
        )
    } else {
        (
            card_surface_color(),
            egui::Stroke::new(1.0, inner_separator_color()),
            dim_text_color(),
        )
    };

    let button = egui::Button::new(egui::RichText::new(label).color(text_color))
        .rounding(egui::Rounding::same(12.0))
        .fill(fill)
        .stroke(stroke);

    if ui.add(button).clicked() {
        *enabled = !*enabled;
    }
}
