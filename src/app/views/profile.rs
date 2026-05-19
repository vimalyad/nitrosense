use eframe::egui;

use crate::app::formatting::{display_profile_name, fallback_profile_names};
use crate::app::NitroSenseApp;
use crate::ui::theme::{accent_color, panel_frame};

impl NitroSenseApp {
    pub(in crate::app::views) fn show_power_profile(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Power Profile")
                        .strong()
                        .color(accent_color()),
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
}
