use eframe::egui;

use super::{AppTab, NitroSenseApp};
use crate::app::formatting::{
    display_profile_name, fallback_profile_names, format_pwm_state, format_rpm, format_temperature,
    format_voltage,
};
use crate::graph::show_graph;
use crate::ui::theme::{
    accent_color, card_surface_color, critical_color, dim_text_color, inner_separator_color,
    panel_frame, readout_color, stat_card_frame, warm_color, warning_color,
};
use crate::ui::widgets::{compact_metric, fan_dashboard_panel, fan_slider_row, nav_button};

impl NitroSenseApp {
    pub(super) fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("NitroSense")
                        .size(32.0)
                        .color(egui::Color32::WHITE),
                );
                ui.label(
                    egui::RichText::new("Fan & Thermal Monitor")
                        .size(11.0)
                        .color(dim_text_color()),
                );
            });
        });
        ui.add_space(8.0);
        ui.painter().line_segment(
            [
                ui.cursor().min,
                egui::pos2(ui.available_rect_before_wrap().right(), ui.cursor().min.y),
            ],
            egui::Stroke::new(3.0, accent_color()),
        );
    }

    pub(super) fn show_navigation(&mut self, ui: &mut egui::Ui) {
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("NitroSense")
                        .size(21.0)
                        .strong()
                        .color(accent_color()),
                );
                ui.label(
                    egui::RichText::new("Fan & Thermal Monitor")
                        .size(10.5)
                        .color(egui::Color32::from_rgb(150, 156, 162)),
                );
            });
        });

        ui.add_space(28.0);
        nav_button(ui, &mut self.active_tab, AppTab::Overview, "Monitoring");
        nav_button(ui, &mut self.active_tab, AppTab::Graph, "Temperature");
        nav_button(ui, &mut self.active_tab, AppTab::FanControl, "Fan Control");

        let footer_gap = (ui.available_height() - 104.0).max(16.0);
        ui.add_space(footer_gap);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Current profile")
                    .size(10.5)
                    .color(dim_text_color()),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    self.sensor_data()
                        .active_power_profile
                        .as_deref()
                        .unwrap_or("profile unavailable"),
                )
                .size(12.5)
                .color(egui::Color32::from_rgb(200, 206, 212)),
            );
        });
        ui.add_space(12.0);
    }

    pub(super) fn show_status_strip(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.set_min_height(52.0);
            ui.horizontal_wrapped(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(124.0, 40.0),
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        compact_metric(
                            ui,
                            "CPU",
                            format_temperature(self.sensor_data().cpu_package_temp_celsius),
                        );
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(124.0, 40.0),
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        compact_metric(
                            ui,
                            "GPU",
                            format_temperature(self.sensor_data().nvidia_gpu_temp_celsius),
                        );
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(124.0, 40.0),
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        compact_metric(ui, "CPU Fan", format_rpm(self.sensor_data().cpu_fan_rpm));
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(124.0, 40.0),
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        compact_metric(ui, "GPU Fan", format_rpm(self.sensor_data().gpu_fan_rpm));
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(124.0, 40.0),
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        compact_metric(
                            ui,
                            "Profile",
                            self.sensor_data()
                                .active_power_profile
                                .clone()
                                .unwrap_or_else(|| "Unavailable".to_owned()),
                        );
                    },
                );
            });
        });
    }

    pub(super) fn show_power_profile(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
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
                    "CPU Fan",
                    format_rpm(self.sensor_data().cpu_fan_rpm),
                    "Fan 1",
                );
                ui.end_row();

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
                ui.end_row();
            });
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: String, detail: &str) {
        stat_card_frame().show(ui, |ui| {
            ui.set_min_width(188.0);
            ui.set_max_width(188.0);
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

    pub(super) fn show_active_tab(&mut self, ui: &mut egui::Ui) {
        match self.active_tab {
            AppTab::Overview => self.show_overview_tab(ui),
            AppTab::Graph => self.show_graph_tab(ui),
            AppTab::FanControl => self.show_fan_control_tab(ui),
        }
    }

    fn show_overview_tab(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_source("overview_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("Monitoring");
                    ui.add_space(8.0);
                    self.show_stats(ui);
                });

                ui.add_space(18.0);

                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("Cooling");
                    ui.add_space(8.0);
                    fan_dashboard_panel(ui, "CPU Fan", self.sensor_data().cpu_fan_rpm);
                    ui.add_space(8.0);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(310.0, 1.0), egui::Sense::hover());
                    ui.painter().line_segment(
                        [rect.left_center(), rect.right_center()],
                        egui::Stroke::new(1.0, inner_separator_color()),
                    );
                    ui.add_space(8.0);
                    fan_dashboard_panel(ui, "GPU Fan", self.sensor_data().gpu_fan_rpm);
                    ui.add_space(8.0);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(310.0, 1.0), egui::Sense::hover());
                    ui.painter().line_segment(
                        [rect.left_center(), rect.right_center()],
                        egui::Stroke::new(1.0, inner_separator_color()),
                    );
                    ui.add_space(8.0);
                    stat_card_frame().show(ui, |ui| {
                        ui.set_min_width(282.0);
                        ui.label(
                            egui::RichText::new("Battery")
                                .size(11.0)
                                .strong()
                                .color(accent_color()),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(format_voltage(self.sensor_data().battery_voltage))
                                .size(20.0)
                                .color(readout_color()),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new("BAT1")
                                .size(10.5)
                                .color(dim_text_color()),
                        );
                    });
                });
            });
    }

    pub(super) fn show_app_toast(&self, context: &egui::Context) {
        let (message, alpha, border_color) = if let (Some(status), Some(shown_at)) =
            (&self.notification_status, self.notification_status_at)
        {
            let elapsed = shown_at.elapsed().as_secs_f32();
            let alpha = if elapsed <= 1.4 {
                1.0
            } else {
                ((2.0 - elapsed) / 0.6).clamp(0.0, 1.0)
            };
            (status.clone(), alpha, accent_color())
        } else if let Some(error) = &self.sensor_snapshot.last_error {
            (
                format!("Sensor issue: {error}"),
                1.0,
                egui::Color32::from_rgb(180, 90, 40),
            )
        } else {
            return;
        };

        if alpha <= 0.0 {
            return;
        }

        let screen = context.screen_rect();
        let width = 420.0;
        let position = egui::pos2(screen.right() - width - 20.0, screen.bottom() - 72.0);
        let fill_alpha = (235.0 * alpha) as u8;
        let text_alpha = (255.0 * alpha) as u8;
        let border_color = egui::Color32::from_rgba_premultiplied(
            border_color.r(),
            border_color.g(),
            border_color.b(),
            text_alpha,
        );

        egui::Area::new("app_toast".into())
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .show(context, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(
                        24, 26, 30, fill_alpha,
                    ))
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.set_width(width - 28.0);
                        ui.label(egui::RichText::new(message).size(12.0).color(
                            egui::Color32::from_rgba_premultiplied(235, 238, 242, text_alpha),
                        ));
                    });
            });
    }

    fn show_graph_tab(&mut self, ui: &mut egui::Ui) {
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

    fn show_fan_control_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fan Control");
        ui.add_space(8.0);

        panel_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(path) = &self.fan_control_status.acer_hwmon_path {
                ui.label(
                    egui::RichText::new(format!("Acer hwmon: {}", path.display()))
                        .monospace()
                        .color(dim_text_color()),
                );
            } else {
                ui.colored_label(warning_color(), "Acer hwmon adapter not found.");
            }

            if !self.fan_control_status.can_control() {
                ui.colored_label(
                    warning_color(),
                    "PWM controls are not available for both fans.",
                );
            }

            ui.add_space(8.0);
            pwm_status_row(
                ui,
                "CPU PWM",
                self.fan_control_status.cpu_pwm,
                self.fan_control_status.cpu_pwm_enable,
            );
            pwm_status_row(
                ui,
                "GPU PWM",
                self.fan_control_status.gpu_pwm,
                self.fan_control_status.gpu_pwm_enable,
            );
        });

        ui.add_space(10.0);
        panel_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let cpu_fan_rpm = self.sensor_data().cpu_fan_rpm;
            let gpu_fan_rpm = self.sensor_data().gpu_fan_rpm;
            let controls_enabled = self.fan_control_status.can_control();

            ui.add_space(4.0);
            let cpu_changed = fan_slider_row(
                ui,
                "CPU Fan",
                &mut self.cpu_fan_percent,
                cpu_fan_rpm,
                controls_enabled,
            );
            ui.add_space(4.0);
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
                    .add_enabled(
                        controls_enabled,
                        egui::Button::new(
                            egui::RichText::new("Auto").color(accent_color()).strong(),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, accent_color())),
                    )
                    .clicked()
                {
                    self.restore_auto_fan_control();
                }
                ui.label(
                    egui::RichText::new("Returns fans to firmware automatic control")
                        .size(10.5)
                        .color(dim_text_color()),
                );
            });
        });

        ui.add_space(10.0);
        self.show_power_profile(ui);

        if let Some(message) = &self.fan_control_message {
            ui.add_space(8.0);
            panel_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(message);
                });
            });
        }
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

fn pwm_status_row(ui: &mut egui::Ui, label: &str, pwm: Option<u8>, mode: Option<u8>) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [72.0, 20.0],
            egui::Label::new(egui::RichText::new(label).color(dim_text_color())),
        );
        ui.add_sized(
            [80.0, 20.0],
            egui::Label::new(egui::RichText::new(pwm_value_label(pwm)).color(readout_color())),
        );
        ui.label(egui::RichText::new(pwm_mode_label(mode)).color(dim_text_color()));
    });
}

fn pwm_value_label(pwm: Option<u8>) -> String {
    format_pwm_state(pwm, None)
}

fn pwm_mode_label(mode: Option<u8>) -> String {
    format_pwm_state(None, mode).to_lowercase()
}

fn graph_toggle_chip(ui: &mut egui::Ui, label: &str, enabled: &mut bool) {
    let (fill, stroke, text_color) = if *enabled {
        (
            egui::Color32::from_rgb(60, 18, 22),
            egui::Stroke::new(1.0, accent_color()),
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
