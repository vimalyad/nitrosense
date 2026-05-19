use eframe::egui;

use crate::app::formatting::format_pwm_state;
use crate::app::NitroSenseApp;
use crate::ui::theme::{accent_color, dim_text_color, panel_frame, readout_color, warning_color};
use crate::ui::widgets::fan_slider_row;

impl NitroSenseApp {
    pub(in crate::app::views) fn show_fan_control_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fan Control");
        ui.add_space(8.0);

        constrained_fan_control_panel(ui, |ui| self.show_fan_status_panel(ui));

        ui.add_space(10.0);
        constrained_fan_control_panel(ui, |ui| self.show_fan_slider_panel(ui));

        ui.add_space(10.0);
        constrained_fan_control_panel(ui, |ui| self.show_power_profile(ui));

        if let Some(message) = visible_fan_control_message(self.fan_control_message.as_deref()) {
            ui.add_space(8.0);
            constrained_fan_control_panel(ui, |ui| show_fan_message(ui, message));
        }
    }

    fn show_fan_status_panel(&self, ui: &mut egui::Ui) {
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
    }

    fn show_fan_slider_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let cpu_fan_rpm = self.sensor_data().cpu_fan_rpm;
            let gpu_fan_rpm = self.sensor_data().gpu_fan_rpm;
            let controls_enabled = self.fan_control_status.can_control();

            let mut cpu_changed = false;
            let mut gpu_changed = false;
            ui.horizontal(|ui| {
                let slider_width = (ui.available_width() - 82.0).max(260.0);
                ui.vertical(|ui| {
                    ui.set_width(slider_width);
                    ui.add_space(4.0);
                    cpu_changed = fan_slider_row(
                        ui,
                        "CPU Fan",
                        &mut self.cpu_fan_percent,
                        cpu_fan_rpm,
                        controls_enabled,
                    );
                    ui.add_space(4.0);
                    gpu_changed = fan_slider_row(
                        ui,
                        "GPU Fan",
                        &mut self.gpu_fan_percent,
                        gpu_fan_rpm,
                        controls_enabled,
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                });
            });

            if controls_enabled && (cpu_changed || gpu_changed) {
                self.schedule_fan_speed_apply(ui.ctx());
            }
        });
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

fn constrained_fan_control_panel(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        let panel_width = (ui.available_width() - 20.0).max(360.0);
        ui.allocate_ui_with_layout(
            egui::vec2(panel_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(panel_width);
                content(ui);
            },
        );
    });
}

fn show_fan_message(ui: &mut egui::Ui, message: &str) {
    panel_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(message);
        });
    });
}

fn visible_fan_control_message(message: Option<&str>) -> Option<&str> {
    message.filter(|value| *value != "Fan control authorized for this session.")
}
