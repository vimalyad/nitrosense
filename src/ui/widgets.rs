use eframe::egui;
use std::time::Duration;

use crate::app::formatting::format_rpm;
use crate::app::AppTab;
use crate::ui::theme::{
    accent_color, card_surface_color, dim_text_color, panel_frame, readout_color, sidebar_color,
    warm_color, warning_color,
};

pub fn nav_button(ui: &mut egui::Ui, active_tab: &mut AppTab, tab: AppTab, label: &str) {
    let active = *active_tab == tab;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(144.0, 38.0), egui::Sense::click());
    let fill = match (active, response.hovered()) {
        (true, _) => egui::Color32::from_rgb(22, 24, 29),
        (false, true) => egui::Color32::from_rgb(18, 20, 25),
        (false, false) => sidebar_color(),
    };
    let text_color = if active {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(185, 190, 195)
    };

    ui.painter()
        .rect_filled(rect, egui::Rounding::same(4.0), fill);
    if active {
        let bar_rect = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.left() + 3.0, rect.bottom()),
        );
        ui.painter().rect_filled(bar_rect, 0.0, accent_color());
    }
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        text_color,
    );

    if response.clicked() {
        *active_tab = tab;
    }
    ui.add_space(6.0);
}

pub fn fan_dashboard_panel(ui: &mut egui::Ui, label: &str, rpm: Option<u32>) {
    ui.ctx().request_repaint_after(Duration::from_millis(16));
    panel_frame().show(ui, |ui| {
        ui.set_min_width(262.0);
        ui.horizontal(|ui| {
            draw_fan_badge(ui, rpm);
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(11.0)
                        .strong()
                        .color(accent_color()),
                );
                ui.label(
                    egui::RichText::new(format_rpm(rpm))
                        .size(22.0)
                        .color(readout_color()),
                );
                ui.add_space(6.0);
                fan_activity_bar(ui, rpm);
            });
        });
    });
}

pub fn fan_activity_bar(ui: &mut egui::Ui, rpm: Option<u32>) {
    let fraction = rpm
        .map(|value| (value as f32 / 6_500.0).clamp(0.0, 1.0))
        .unwrap_or(0.0);

    let width = ui.available_width().max(1.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
    let rounding = egui::Rounding::same(4.0);
    let fill_color = if fraction < 0.40 {
        egui::Color32::from_rgb(40, 160, 90)
    } else if fraction <= 0.75 {
        warm_color()
    } else {
        warning_color()
    };

    ui.painter()
        .rect_filled(rect, rounding, card_surface_color());

    if fraction > 0.0 {
        let filled_rect = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.left() + rect.width() * fraction, rect.bottom()),
        );
        ui.painter().rect_filled(filled_rect, rounding, fill_color);
    }
}

pub fn fan_slider_row(
    ui: &mut egui::Ui,
    label: &str,
    percent: &mut u8,
    rpm: Option<u32>,
    enabled: bool,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.set_min_height(32.0);
        ui.add_sized(
            [68.0, 22.0],
            egui::Label::new(egui::RichText::new(label).color(dim_text_color())),
        );

        let mut value = *percent as f32;
        let slider_width = (ui.available_width() - 42.0 - 72.0 - 12.0).max(180.0);
        let response = ui.add_enabled_ui(enabled, |ui| {
            ui.add_sized(
                [slider_width, 20.0],
                egui::Slider::new(&mut value, 0.0..=100.0).show_value(false),
            )
        });
        let response = response.inner;
        let next_percent = value.round().clamp(0.0, 100.0) as u8;
        if response.changed() && *percent != next_percent {
            changed = true;
        }
        *percent = next_percent;

        let (badge_rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 22.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(badge_rect, egui::Rounding::same(4.0), card_surface_color());
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}%", *percent),
            egui::FontId::proportional(11.5),
            readout_color(),
        );

        ui.add_sized(
            [72.0, 20.0],
            egui::Label::new(
                egui::RichText::new(format_rpm(rpm))
                    .size(11.5)
                    .color(dim_text_color()),
            ),
        );
    });

    changed
}

fn draw_fan_badge(ui: &mut egui::Ui, rpm: Option<u32>) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(72.0, 72.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = 30.0;
    let speed = rpm
        .map(|value| (value as f32 / 7_500.0).clamp(0.15, 1.0))
        .unwrap_or(0.15);
    let elapsed = ui.ctx().input(|input| input.time) as f32;
    let rotation = elapsed * (3.0 + speed * 10.0);

    painter.circle_filled(center, radius, egui::Color32::from_rgb(24, 26, 30));
    painter.circle_stroke(center, radius, egui::Stroke::new(2.0, accent_color()));

    for blade in 0..3 {
        let angle = blade as f32 * std::f32::consts::TAU / 3.0 + rotation;
        let tip = center + egui::vec2(angle.cos(), angle.sin()) * 24.0;
        let side = center + egui::vec2((angle + 1.95).cos(), (angle + 1.95).sin()) * 10.0;
        painter.line_segment(
            [center, tip],
            egui::Stroke::new(6.0, egui::Color32::from_rgb(220, 34, 44)),
        );
        painter.line_segment(
            [center, side],
            egui::Stroke::new(3.0, egui::Color32::from_rgb(125, 130, 136)),
        );
    }

    painter.circle_filled(center, 6.0, egui::Color32::WHITE);
}
