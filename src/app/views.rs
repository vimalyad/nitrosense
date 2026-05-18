use eframe::egui;

use super::{AppTab, NitroSenseApp};
use crate::app::formatting::{
    display_profile_name, fallback_profile_names, format_pwm_state, format_rpm, format_temperature,
    format_voltage,
};
use crate::graph::show_graph;
use crate::ui::theme::{accent_color, panel_frame, warning_color};
use crate::ui::widgets::{compact_metric, fan_dashboard_panel, fan_slider_row, nav_button};

impl NitroSenseApp {
    pub(super) fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("NitroSense")
                    .size(30.0)
                    .color(egui::Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("Acer Nitro AN515-58")
                        .strong()
                        .color(egui::Color32::from_rgb(205, 210, 215)),
                );
            });
        });
        ui.add_space(8.0);
        ui.painter().line_segment(
            [
                ui.cursor().min,
                egui::pos2(ui.available_rect_before_wrap().right(), ui.cursor().min.y),
            ],
            egui::Stroke::new(2.0, accent_color()),
        );
    }

    pub(super) fn show_navigation(&mut self, ui: &mut egui::Ui) {
        ui.add_space(18.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("NITRO")
                    .size(26.0)
                    .strong()
                    .color(accent_color()),
            );
            ui.label(
                egui::RichText::new("AN515-58")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 156, 162)),
            );
        });

        ui.add_space(28.0);
        nav_button(ui, &mut self.active_tab, AppTab::Overview, "Monitoring");
        nav_button(ui, &mut self.active_tab, AppTab::Graph, "Temperature");
        nav_button(ui, &mut self.active_tab, AppTab::FanControl, "Fan Control");

        let footer_gap = (ui.available_height() - 52.0).max(16.0);
        ui.add_space(footer_gap);
        ui.label(egui::RichText::new("Current profile").small().weak());
        ui.label(
            egui::RichText::new(
                self.sensor_data()
                    .active_power_profile
                    .as_deref()
                    .unwrap_or("profile unavailable"),
            )
            .color(egui::Color32::from_rgb(190, 196, 202)),
        );
    }

    pub(super) fn show_status_strip(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                compact_metric(
                    ui,
                    "CPU",
                    format_temperature(self.sensor_data().cpu_package_temp_celsius),
                );
                compact_metric(
                    ui,
                    "GPU",
                    format_temperature(self.sensor_data().nvidia_gpu_temp_celsius),
                );
                compact_metric(ui, "CPU Fan", format_rpm(self.sensor_data().cpu_fan_rpm));
                compact_metric(ui, "GPU Fan", format_rpm(self.sensor_data().gpu_fan_rpm));
                compact_metric(
                    ui,
                    "Profile",
                    self.sensor_data()
                        .active_power_profile
                        .clone()
                        .unwrap_or_else(|| "Unavailable".to_owned()),
                );
            });
        });
    }

    pub(super) fn show_power_profile(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Power Profile")
                        .strong()
                        .color(accent_color()),
                );
                ui.label(
                    self.sensor_data()
                        .active_power_profile
                        .as_deref()
                        .unwrap_or("Unavailable"),
                );
            });

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let choices: Vec<String> = if self.profile_choices.is_empty() {
                    fallback_profile_names()
                } else {
                    self.profile_choices
                        .iter()
                        .map(|profile| profile.name.clone())
                        .collect()
                };

                for profile_name in choices {
                    let active = self
                        .sensor_data()
                        .active_power_profile
                        .as_deref()
                        .map(|current| current.eq_ignore_ascii_case(&profile_name))
                        .unwrap_or(false);

                    let available = self.profile_choices.is_empty()
                        || self
                            .profile_choices
                            .iter()
                            .any(|profile| profile.name == profile_name);

                    if ui
                        .add_enabled(
                            available,
                            egui::SelectableLabel::new(active, display_profile_name(&profile_name)),
                        )
                        .clicked()
                    {
                        self.set_power_profile(profile_name);
                    }
                }
            });

            if let Some(status) = &self.profile_status {
                ui.add_space(6.0);
                ui.label(status);
            }
        });
    }

    pub(super) fn show_stats(&self, ui: &mut egui::Ui) {
        egui::Grid::new("stats_grid")
            .num_columns(3)
            .spacing([12.0, 12.0])
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
                self.stat_card(
                    ui,
                    "NVMe",
                    format_temperature(self.sensor_data().nvme_temp_celsius),
                    "Storage",
                );
                ui.end_row();

                self.stat_card(
                    ui,
                    "CPU Fan",
                    format_rpm(self.sensor_data().cpu_fan_rpm),
                    "Fan 1",
                );
                self.stat_card(
                    ui,
                    "GPU Fan",
                    format_rpm(self.sensor_data().gpu_fan_rpm),
                    "Fan 2",
                );
                self.stat_card(
                    ui,
                    "Battery",
                    format_voltage(self.sensor_data().battery_voltage),
                    "BAT1",
                );
                ui.end_row();

                self.stat_card(
                    ui,
                    "Profile",
                    self.sensor_data()
                        .active_power_profile
                        .clone()
                        .unwrap_or_else(|| "Unavailable".to_owned()),
                    "Platform",
                );
                ui.label("");
                ui.label("");
                ui.end_row();
            });
    }

    pub(super) fn show_polling_status(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.sensor_snapshot.last_error {
            ui.add_space(8.0);
            ui.colored_label(
                egui::Color32::from_rgb(180, 90, 40),
                format!("Sensor polling issue: {error}"),
            );
        }
    }

    pub(super) fn show_tray_status(&self, ui: &mut egui::Ui) {
        if self.window_hidden_to_tray {
            ui.add_space(8.0);
            ui.label("Window hidden to tray.");
        }
    }

    pub(super) fn show_notification_status(&self, ui: &mut egui::Ui) {
        if let Some(status) = &self.notification_status {
            ui.add_space(8.0);
            ui.label(status);
        }
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: String, detail: &str) {
        panel_frame()
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.label(egui::RichText::new(title).strong().color(accent_color()));
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(value)
                        .size(24.0)
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.label(egui::RichText::new(detail).small().weak());
            });
    }

    pub(super) fn show_active_tab(&mut self, ui: &mut egui::Ui) {
        match self.active_tab {
            AppTab::Overview => self.show_overview_tab(ui),
            AppTab::Graph => self.show_graph_tab(ui),
            AppTab::FanControl => self.show_fan_control_tab(ui),
        }
    }

    fn show_overview_tab(&self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            columns[0].heading("Monitoring");
            columns[0].add_space(8.0);
            self.show_stats(&mut columns[0]);

            columns[1].heading("Cooling");
            columns[1].add_space(8.0);
            fan_dashboard_panel(&mut columns[1], "CPU Fan", self.sensor_data().cpu_fan_rpm);
            columns[1].add_space(10.0);
            fan_dashboard_panel(&mut columns[1], "GPU Fan", self.sensor_data().gpu_fan_rpm);
            columns[1].add_space(10.0);
            panel_frame().show(&mut columns[1], |ui| {
                ui.label(
                    egui::RichText::new("Battery")
                        .strong()
                        .color(accent_color()),
                );
                ui.label(format_voltage(self.sensor_data().battery_voltage));
            });
        });
    }

    fn show_graph_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Temperature");
        ui.add_space(8.0);
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.graph_visibility.cpu_temp, "CPU Temp");
                ui.checkbox(&mut self.graph_visibility.gpu_temp, "GPU Temp");
            });
            ui.add_space(8.0);
            show_graph(ui, &self.graph_history, &self.graph_visibility);
        });
    }

    fn show_fan_control_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fan Control");
        ui.add_space(8.0);

        panel_frame().show(ui, |ui| {
            if let Some(path) = &self.fan_control_status.acer_hwmon_path {
                ui.label(format!("Acer hwmon: {}", path.display()));
            } else {
                ui.colored_label(warning_color(), "Acer hwmon adapter not found.");
            }

            if !self.fan_control_status.can_control() {
                ui.colored_label(
                    warning_color(),
                    "PWM controls are not available for both fans.",
                );
            }

            ui.label(format!(
                "CPU PWM: {} | GPU PWM: {}",
                format_pwm_state(
                    self.fan_control_status.cpu_pwm,
                    self.fan_control_status.cpu_pwm_enable
                ),
                format_pwm_state(
                    self.fan_control_status.gpu_pwm,
                    self.fan_control_status.gpu_pwm_enable
                )
            ));
        });

        ui.add_space(10.0);
        panel_frame().show(ui, |ui| {
            let cpu_fan_rpm = self.sensor_data().cpu_fan_rpm;
            let gpu_fan_rpm = self.sensor_data().gpu_fan_rpm;
            let controls_enabled = self.fan_control_status.can_control();

            let cpu_changed = fan_slider_row(
                ui,
                "CPU Fan",
                &mut self.cpu_fan_percent,
                cpu_fan_rpm,
                controls_enabled,
            );
            ui.add_space(8.0);
            let gpu_changed = fan_slider_row(
                ui,
                "GPU Fan",
                &mut self.gpu_fan_percent,
                gpu_fan_rpm,
                controls_enabled,
            );

            if controls_enabled && (cpu_changed || gpu_changed) {
                self.schedule_fan_speed_apply(ui.ctx());
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(controls_enabled, egui::Button::new("Auto"))
                    .clicked()
                {
                    self.restore_auto_fan_control();
                }
            });
        });

        ui.add_space(10.0);
        self.show_power_profile(ui);

        if let Some(message) = &self.fan_control_message {
            ui.add_space(8.0);
            ui.label(message);
        }
    }
}
