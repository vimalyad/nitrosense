use eframe::egui;

pub fn apply_nitro_style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = app_background_color();
    visuals.panel_fill = app_background_color();
    visuals.widgets.active.bg_fill = accent_color();
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(86, 26, 30);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 38, 43);
    visuals.selection.bg_fill = accent_color();
    context.set_visuals(visuals);
}

pub fn panel_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(panel_color())
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(52, 56, 62)))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
}

pub fn app_background_color() -> egui::Color32 {
    egui::Color32::from_rgb(15, 17, 21)
}

pub fn sidebar_color() -> egui::Color32 {
    egui::Color32::from_rgb(10, 11, 14)
}

pub fn accent_color() -> egui::Color32 {
    egui::Color32::from_rgb(226, 31, 42)
}

pub fn warning_color() -> egui::Color32 {
    egui::Color32::from_rgb(235, 150, 60)
}

fn panel_color() -> egui::Color32 {
    egui::Color32::from_rgb(28, 31, 36)
}
