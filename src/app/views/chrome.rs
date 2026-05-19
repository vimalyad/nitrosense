use eframe::egui;

use crate::app::{AppTab, NitroSenseApp};
use crate::ui::theme::{accent_color, dim_text_color};
use crate::ui::widgets::nav_button;

impl NitroSenseApp {
    pub(in crate::app) fn show_header(&self, ui: &mut egui::Ui) {
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

    pub(in crate::app) fn show_navigation(&mut self, ui: &mut egui::Ui) {
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

        let footer_gap = (ui.available_height() - 154.0).max(12.0);
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
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("created by")
                    .size(10.5)
                    .color(dim_text_color()),
            );
            ui.label(
                egui::RichText::new("vimalyad")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(200, 206, 212)),
            );
        });
        ui.add_space(12.0);
    }

    pub(in crate::app) fn show_active_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                match self.active_tab {
                    AppTab::Overview => self.show_overview_tab(ui),
                    AppTab::Graph => self.show_graph_tab(ui),
                    AppTab::FanControl => self.show_fan_control_tab(ui),
                }
            });
        });
    }

    pub(in crate::app) fn show_app_toast(&self, context: &egui::Context) {
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
}
