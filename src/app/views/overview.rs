use eframe::egui;

use crate::app::formatting::format_temperature;
use crate::app::NitroSenseApp;
use crate::ui::theme::{
    accent_color, critical_color, dim_text_color, inner_separator_color, readout_color,
    stat_card_frame, warm_color, warning_color,
};
use crate::ui::widgets::fan_dashboard_panel;

impl NitroSenseApp {
    pub(in crate::app::views) fn show_overview_tab(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            ui.heading("Thermals");
            ui.add_space(8.0);
            self.show_stats(ui);
        });

        ui.add_space(18.0);

        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 20.0).max(300.0));
            ui.heading("Cooling");
            ui.add_space(8.0);
            fan_dashboard_panel(ui, "CPU Fan", self.sensor_data().cpu_fan_rpm);
            ui.add_space(8.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(310.0, 1.0), egui::Sense::hover());
            ui.painter().line_segment(
                [rect.left_center(), rect.right_center()],
                egui::Stroke::new(1.0, inner_separator_color()),
            );
            ui.add_space(8.0);
            fan_dashboard_panel(ui, "GPU Fan", self.sensor_data().gpu_fan_rpm);
            ui.add_space(8.0);
        });
    }

    fn show_stats(&self, ui: &mut egui::Ui) {
        egui::Grid::new("stats_grid")
            .num_columns(2)
            .spacing([14.0, 14.0])
            .show(ui, |ui| {
                self.stat_card(
                    ui,
                    "CPU",
                    format_temperature(self.sensor_data().cpu_package_temp_celsius),
                    "Package",
                );
                self.stat_card(
                    ui,
                    "NVIDIA GPU",
                    format_temperature(self.sensor_data().nvidia_gpu_temp_celsius),
                    "Discrete",
                );
                ui.end_row();

                self.stat_card(
                    ui,
                    "NVMe",
                    format_temperature(self.sensor_data().nvme_temp_celsius),
                    "Storage",
                );
                self.stat_card(
                    ui,
                    "Profile",
                    self.sensor_data()
                        .active_power_profile
                        .clone()
                        .unwrap_or_else(|| "Unavailable".to_owned()),
                    "Platform",
                );
                ui.end_row();
            });
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: String, detail: &str) {
        stat_card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(11.0)
                    .strong()
                    .color(accent_color()),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(value.clone())
                    .size(26.0)
                    .color(stat_value_color(&value)),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(detail)
                    .size(10.5)
                    .color(dim_text_color()),
            );
        });
    }
}

fn stat_value_color(value: &str) -> egui::Color32 {
    if value.ends_with(" C") {
        if let Ok(value_celsius) = value.trim_end_matches(" C").parse::<f32>() {
            return temperature_color(value_celsius);
        }
    }

    readout_color()
}

fn temperature_color(value_celsius: f32) -> egui::Color32 {
    if value_celsius >= 90.0 {
        critical_color()
    } else if value_celsius >= 80.0 {
        warning_color()
    } else if value_celsius >= 70.0 {
        warm_color()
    } else {
        readout_color()
    }
}
