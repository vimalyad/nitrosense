#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayTemperatureState {
    Normal,
    Warm,
    Hot,
}

pub fn state_for_cpu_temp(cpu_temp_celsius: Option<f32>) -> TrayTemperatureState {
    match cpu_temp_celsius {
        Some(temp) if temp >= 85.0 => TrayTemperatureState::Hot,
        Some(temp) if temp >= 70.0 => TrayTemperatureState::Warm,
        _ => TrayTemperatureState::Normal,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(feature = "tray"), allow(dead_code))]
pub enum TrayAction {
    ShowWindow,
    Quit,
    SetProfile(String),
}

#[cfg(feature = "tray")]
mod platform {
    use std::collections::HashMap;

    use tray_icon::{
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
        Icon, TrayIcon, TrayIconBuilder,
    };

    use super::{TrayAction, TrayTemperatureState};

    pub struct TrayController {
        tray_icon: Option<TrayIcon>,
        current_state: TrayTemperatureState,
        show_window_id: tray_icon::menu::MenuId,
        quit_id: tray_icon::menu::MenuId,
        profile_ids: HashMap<tray_icon::menu::MenuId, String>,
    }

    impl TrayController {
        pub fn new(profile_names: &[String]) -> Self {
            let menu = Menu::new();
            let show_window = MenuItem::with_id("show-window", "Show Window", true, None);
            let quit = MenuItem::with_id("quit", "Quit", true, None);
            let mut profile_items = Vec::new();
            let mut profile_ids = HashMap::new();

            for profile_name in profile_names {
                let id = format!("profile:{profile_name}");
                let item = MenuItem::with_id(
                    id,
                    format!("Profile: {}", display_profile_name(profile_name)),
                    true,
                    None,
                );
                profile_ids.insert(item.id().clone(), profile_name.clone());
                profile_items.push(item);
            }

            let _ = menu.append(&show_window);
            let _ = menu.append(&PredefinedMenuItem::separator());
            for item in &profile_items {
                let _ = menu.append(item);
            }
            let _ = menu.append(&PredefinedMenuItem::separator());
            let _ = menu.append(&quit);

            let tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("NitroSense")
                .with_title("NitroSense")
                .with_icon(icon_for_state(TrayTemperatureState::Normal))
                .build()
                .ok();

            Self {
                tray_icon,
                current_state: TrayTemperatureState::Normal,
                show_window_id: show_window.id().clone(),
                quit_id: quit.id().clone(),
                profile_ids,
            }
        }

        pub fn set_temperature_state(&mut self, state: TrayTemperatureState) {
            if self.current_state == state {
                return;
            }

            self.current_state = state;

            if let Some(tray_icon) = &self.tray_icon {
                let _ = tray_icon.set_icon(Some(icon_for_state(state)));
            }
        }

        pub fn set_tooltip(&self, tooltip: String) {
            if let Some(tray_icon) = &self.tray_icon {
                let _ = tray_icon.set_tooltip(Some(tooltip));
            }
        }

        pub fn is_available(&self) -> bool {
            self.tray_icon.is_some()
        }

        pub fn poll_action(&self) -> Option<TrayAction> {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == self.show_window_id {
                    return Some(TrayAction::ShowWindow);
                }

                if event.id == self.quit_id {
                    return Some(TrayAction::Quit);
                }

                if let Some(profile_name) = self.profile_ids.get(&event.id) {
                    return Some(TrayAction::SetProfile(profile_name.clone()));
                }
            }

            None
        }
    }

    fn icon_for_state(state: TrayTemperatureState) -> Icon {
        let color = match state {
            TrayTemperatureState::Normal => [40, 170, 95, 255],
            TrayTemperatureState::Warm => [230, 145, 45, 255],
            TrayTemperatureState::Hot => [215, 65, 65, 255],
        };
        let mut rgba = Vec::with_capacity(32 * 32 * 4);

        for y in 0..32 {
            for x in 0..32 {
                let dx = x as i32 - 16;
                let dy = y as i32 - 16;
                let inside = dx * dx + dy * dy <= 14 * 14;

                if inside {
                    rgba.extend_from_slice(&color);
                } else {
                    rgba.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }

        Icon::from_rgba(rgba, 32, 32).expect("generated tray icon must be valid")
    }

    fn display_profile_name(profile_name: &str) -> String {
        profile_name
            .split(['-', '_'])
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(not(feature = "tray"))]
mod platform {
    use super::{TrayAction, TrayTemperatureState};

    #[derive(Default)]
    pub struct TrayController;

    impl TrayController {
        pub fn new(_profile_names: &[String]) -> Self {
            Self
        }

        pub fn set_temperature_state(&mut self, _state: TrayTemperatureState) {}

        pub fn set_tooltip(&self, _tooltip: String) {}

        pub fn is_available(&self) -> bool {
            false
        }

        pub fn poll_action(&self) -> Option<TrayAction> {
            None
        }
    }
}

pub use platform::TrayController;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_cpu_temperature_to_tray_state() {
        assert_eq!(state_for_cpu_temp(None), TrayTemperatureState::Normal);
        assert_eq!(state_for_cpu_temp(Some(69.9)), TrayTemperatureState::Normal);
        assert_eq!(state_for_cpu_temp(Some(70.0)), TrayTemperatureState::Warm);
        assert_eq!(state_for_cpu_temp(Some(85.0)), TrayTemperatureState::Hot);
    }
}
