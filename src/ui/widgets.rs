use eframe::egui;

use crate::app::formatting::format_rpm;
use crate::app::AppTab;
use crate::ui::theme::{accent_color, panel_frame, sidebar_color};

pub fn nav_button(ui: &mut egui::Ui, active_tab: &mut AppTab, tab: AppTab, label: &str) {
    let active = *active_tab == tab;
    let fill = if active {
        egui::Color32::from_rgb(120, 24, 30)
    } else {
        sidebar_color()
    };
    let text_color = if active {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(185, 190, 195)
    };

    let button = egui::Button::new(egui::RichText::new(label).strong().color(text_color))
        .fill(fill)
        .stroke(if active {
            egui::Stroke::new(1.0, accent_color())
        } else {
            egui::Stroke::NONE
        });

    if ui.add_sized([144.0, 38.0], button).clicked() {
        *active_tab = tab;
    }
    ui.add_space(6.0);
}

pub fn compact_metric(ui: &mut egui::Ui, label: &str, value: String) {
    ui.vertical(|ui| {
        ui.set_min_width(112.0);
        ui.label(egui::RichText::new(label).small().weak());
        ui.label(
            egui::RichText::new(value)
                .strong()
                .color(egui::Color32::WHITE),
        );
    });
    ui.separator();
}

pub fn fan_dashboard_panel(ui: &mut egui::Ui, label: &str, rpm: Option<u32>) {
    panel_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            draw_fan_badge(ui, rpm);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(label).strong().color(accent_color()));
                ui.label(
                    egui::RichText::new(format_rpm(rpm))
                        .size(22.0)
                        .color(egui::Color32::WHITE),
                );
                fan_activity_bar(ui, label, rpm);
            });
        });
    });
}

pub fn fan_activity_bar(ui: &mut egui::Ui, label: &str, rpm: Option<u32>) {
    let fraction = rpm
        .map(|value| (value as f32 / 6_500.0).clamp(0.0, 1.0))
        .unwrap_or(0.0);
    let text = format!("{label}: {}", format_rpm(rpm));

    ui.add(
        egui::ProgressBar::new(fraction)
            .desired_width(360.0)
            .text(text),
    );
}

pub fn fan_slider_row(ui: &mut egui::Ui, label: &str, percent: &mut u8, rpm: Option<u32>) {
    ui.horizontal(|ui| {
        ui.set_min_height(32.0);
        ui.label(label);

        let mut value = *percent as f32;
        ui.add(egui::Slider::new(&mut value, 0.0..=100.0).show_value(false));
        *percent = value.round().clamp(0.0, 100.0) as u8;

        ui.label(format!("{}%", *percent));
        ui.label(format_rpm(rpm));
    });
}

fn draw_fan_badge(ui: &mut egui::Ui, rpm: Option<u32>) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(72.0, 72.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = 30.0;
    let speed = rpm
        .map(|value| (value as f32 / 7_500.0).clamp(0.15, 1.0))
        .unwrap_or(0.15);

    painter.circle_filled(center, radius, egui::Color32::from_rgb(24, 26, 30));
    painter.circle_stroke(center, radius, egui::Stroke::new(2.0, accent_color()));

    for blade in 0..3 {
        let angle = blade as f32 * std::f32::consts::TAU / 3.0 + speed;
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
