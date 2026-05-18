use anyhow::{anyhow, Result};
use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::watch;

use crate::config::AppConfig;
use crate::fan_control::{self, FanControlStatus, FanId};
use crate::graph::{show_graph, GraphHistory, GraphVisibility};
use crate::notifications::{send_desktop_notification, ThermalAlertState};
use crate::polling::{spawn_sensor_polling, SensorSnapshot};
use crate::profile::{self, PowerProfile};
use crate::sensors::SensorData;
use crate::tray::{state_for_cpu_temp, TrayAction, TrayController};

pub fn run() -> Result<()> {
    let runtime = Runtime::new()?;
    let sensor_receiver = spawn_sensor_polling(runtime.handle());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 640.0])
            .with_icon(load_window_icon()?),
        ..Default::default()
    };

    eframe::run_native(
        "NitroSense",
        options,
        Box::new(|creation_context| {
            Box::new(NitroSenseApp::new(
                creation_context,
                runtime,
                sensor_receiver,
            ))
        }),
    )
    .map_err(|error| anyhow!("failed to launch NitroSense UI: {error}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Overview,
    Graph,
    FanControl,
}

struct NitroSenseApp {
    _runtime: Runtime,
    sensor_receiver: watch::Receiver<SensorSnapshot>,
    sensor_snapshot: SensorSnapshot,
    graph_history: GraphHistory,
    graph_visibility: GraphVisibility,
    profile_choices: Vec<PowerProfile>,
    profile_status: Option<String>,
    fan_control_status: FanControlStatus,
    cpu_fan_percent: u8,
    gpu_fan_percent: u8,
    fan_control_message: Option<String>,
    app_config: AppConfig,
    thermal_alerts: ThermalAlertState,
    notification_status: Option<String>,
    tray_controller: TrayController,
    window_hidden_to_tray: bool,
    allow_window_close: bool,
    active_tab: AppTab,
}

impl NitroSenseApp {
    fn new(
        _creation_context: &eframe::CreationContext<'_>,
        runtime: Runtime,
        sensor_receiver: watch::Receiver<SensorSnapshot>,
    ) -> Self {
        let sensor_snapshot = sensor_receiver.borrow().clone();

        let mut graph_history = GraphHistory::new();
        graph_history.push(std::time::Instant::now(), &sensor_snapshot.data);

        let profile_choices = profile::read_profile_choices().unwrap_or_default();
        let profile_names = if profile_choices.is_empty() {
            fallback_profile_names()
        } else {
            profile_choices
                .iter()
                .map(|profile| profile.name.clone())
                .collect()
        };

        let mut tray_controller = TrayController::new(&profile_names);
        tray_controller.set_temperature_state(state_for_cpu_temp(
            sensor_snapshot.data.cpu_package_temp_celsius,
        ));
        tray_controller.set_tooltip(tray_tooltip(&sensor_snapshot.data));

        Self {
            _runtime: runtime,
            sensor_receiver,
            sensor_snapshot,
            graph_history,
            graph_visibility: GraphVisibility::default(),
            profile_choices,
            profile_status: None,
            fan_control_status: FanControlStatus::detect(),
            cpu_fan_percent: 50,
            gpu_fan_percent: 50,
            fan_control_message: None,
            app_config: AppConfig::default(),
            thermal_alerts: ThermalAlertState::default(),
            notification_status: None,
            tray_controller,
            window_hidden_to_tray: false,
            allow_window_close: false,
            active_tab: AppTab::Overview,
        }
    }
}

impl eframe::App for NitroSenseApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_sensor_snapshot();
        self.handle_window_close_request(context);
        self.handle_tray_action(context);

        egui::CentralPanel::default().show(context, |ui| {
            ui.add_space(8.0);
            self.show_header(ui);
            ui.add_space(12.0);
            self.show_power_profile(ui);
            ui.add_space(12.0);
            self.show_stats(ui);
            self.show_polling_status(ui);
            self.show_notification_status(ui);
            self.show_tray_status(ui);
            ui.add_space(12.0);
            self.show_tabs(ui);
            ui.separator();
            ui.add_space(8.0);
            self.show_active_tab(ui);
        });
    }
}

impl NitroSenseApp {
    fn refresh_sensor_snapshot(&mut self) {
        if self.sensor_receiver.has_changed().unwrap_or(false) {
            self.sensor_snapshot = self.sensor_receiver.borrow_and_update().clone();
            self.graph_history
                .push(std::time::Instant::now(), &self.sensor_snapshot.data);
            self.update_tray();
            self.process_thermal_alerts();
        }
    }

    fn update_tray(&mut self) {
        self.tray_controller
            .set_temperature_state(state_for_cpu_temp(
                self.sensor_snapshot.data.cpu_package_temp_celsius,
            ));
        self.tray_controller
            .set_tooltip(tray_tooltip(&self.sensor_snapshot.data));
    }

    fn handle_window_close_request(&mut self, context: &egui::Context) {
        if self.allow_window_close || !self.tray_controller.is_available() {
            return;
        }

        if context.input(|input| input.viewport().close_requested()) {
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            context.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.window_hidden_to_tray = true;
        }
    }

    fn handle_tray_action(&mut self, context: &egui::Context) {
        while let Some(action) = self.tray_controller.poll_action() {
            match action {
                TrayAction::ShowWindow => {
                    context.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    context.send_viewport_cmd(egui::ViewportCommand::Focus);
                    self.window_hidden_to_tray = false;
                }
                TrayAction::Quit => {
                    self.allow_window_close = true;
                    context.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                TrayAction::SetProfile(profile_name) => self.set_power_profile(profile_name),
            }
        }
    }

    fn process_thermal_alerts(&mut self) {
        let now = std::time::Instant::now();
        let alerts =
            self.thermal_alerts
                .pending_alerts(&self.sensor_snapshot.data, &self.app_config, now);

        for alert in alerts {
            match send_desktop_notification(&alert) {
                Ok(()) => {
                    self.thermal_alerts.mark_sent(&alert, now);
                    self.notification_status = Some(format!(
                        "{} notification sent: {}",
                        alert.kind.title(),
                        alert.message()
                    ));
                }
                Err(error) => {
                    self.notification_status =
                        Some(format!("Could not send thermal notification: {error}"));
                }
            }
        }
    }

    fn sensor_data(&self) -> &SensorData {
        &self.sensor_snapshot.data
    }

    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("NitroSense");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Acer Nitro AN515-58");
            });
        });
    }

    fn show_power_profile(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Power Profile");
                ui.separator();
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

    fn set_power_profile(&mut self, profile_name: String) {
        match profile::set_active_profile(&profile_name) {
            Ok(()) => {
                self.profile_status = Some(format!(
                    "Requested {} profile.",
                    display_profile_name(&profile_name)
                ));
            }
            Err(error) => {
                self.profile_status = Some(format!(
                    "Could not set {}: {error}",
                    display_profile_name(&profile_name)
                ));
            }
        }
    }

    fn show_stats(&self, ui: &mut egui::Ui) {
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
                    "Intel GPU",
                    format_temperature(self.sensor_data().intel_gpu_temp_celsius),
                    "Integrated",
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
                    "NVMe",
                    format_temperature(self.sensor_data().nvme_temp_celsius),
                    "Storage",
                );
                ui.end_row();

                self.stat_card(
                    ui,
                    "Battery",
                    format_voltage(self.sensor_data().battery_voltage),
                    "BAT1",
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
                ui.label("");
                ui.end_row();
            });
    }

    fn show_polling_status(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.sensor_snapshot.last_error {
            ui.add_space(8.0);
            ui.colored_label(
                egui::Color32::from_rgb(180, 90, 40),
                format!("Sensor polling issue: {error}"),
            );
        }
    }

    fn show_tray_status(&self, ui: &mut egui::Ui) {
        if self.window_hidden_to_tray {
            ui.add_space(8.0);
            ui.label("Window hidden to tray.");
        }
    }

    fn show_notification_status(&self, ui: &mut egui::Ui) {
        if let Some(status) = &self.notification_status {
            ui.add_space(8.0);
            ui.label(status);
        }
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: String, detail: &str) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .rounding(egui::Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.label(egui::RichText::new(title).strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new(value).size(24.0));
                ui.add_space(2.0);
                ui.label(egui::RichText::new(detail).small().weak());
            });
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, AppTab::Overview, "Overview");
            ui.selectable_value(&mut self.active_tab, AppTab::Graph, "Graph");
            ui.selectable_value(&mut self.active_tab, AppTab::FanControl, "Fan Control");
        });
    }

    fn show_active_tab(&mut self, ui: &mut egui::Ui) {
        match self.active_tab {
            AppTab::Overview => self.show_overview_tab(ui),
            AppTab::Graph => self.show_graph_tab(ui),
            AppTab::FanControl => self.show_fan_control_tab(ui),
        }
    }

    fn show_overview_tab(&self, ui: &mut egui::Ui) {
        ui.heading("Overview");
        ui.add_space(8.0);
        fan_activity_bar(ui, "CPU Fan", self.sensor_data().cpu_fan_rpm);
        fan_activity_bar(ui, "GPU Fan", self.sensor_data().gpu_fan_rpm);
        ui.add_space(8.0);
        ui.label(format!(
            "Battery voltage: {}",
            format_voltage(self.sensor_data().battery_voltage)
        ));
    }

    fn show_graph_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Graph");
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.graph_visibility.cpu_temp, "CPU Temp");
            ui.checkbox(&mut self.graph_visibility.gpu_temp, "GPU Temp");
        });
        ui.add_space(8.0);
        show_graph(ui, &self.graph_history, &self.graph_visibility);
    }

    fn show_fan_control_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fan Control");
        ui.add_space(8.0);

        if !self.fan_control_status.nbfc_available {
            ui.colored_label(
                egui::Color32::from_rgb(180, 90, 40),
                "NBFC command not found.",
            );
        }

        if !self.fan_control_status.service_available {
            ui.colored_label(
                egui::Color32::from_rgb(180, 90, 40),
                "nbfc_service is not active.",
            );
        }

        ui.add_space(8.0);
        let cpu_fan_rpm = self.sensor_data().cpu_fan_rpm;
        let gpu_fan_rpm = self.sensor_data().gpu_fan_rpm;

        fan_slider_row(ui, "CPU Fan", &mut self.cpu_fan_percent, cpu_fan_rpm);
        fan_slider_row(ui, "GPU Fan", &mut self.gpu_fan_percent, gpu_fan_rpm);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let controls_enabled =
                self.fan_control_status.nbfc_available && self.fan_control_status.service_available;

            if ui
                .add_enabled(controls_enabled, egui::Button::new("Apply"))
                .clicked()
            {
                self.apply_manual_fan_speeds();
            }

            if ui
                .add_enabled(
                    self.fan_control_status.nbfc_available,
                    egui::Button::new("Auto"),
                )
                .clicked()
            {
                self.restore_auto_fan_control();
            }

            if ui.button("Refresh Status").clicked() {
                self.fan_control_status = FanControlStatus::detect();
                self.fan_control_message = Some("Fan control status refreshed.".to_owned());
            }
        });

        if let Some(message) = &self.fan_control_message {
            ui.add_space(8.0);
            ui.label(message);
        }
    }

    fn apply_manual_fan_speeds(&mut self) {
        let cpu_result = fan_control::set_manual_speed(FanId::Cpu, self.cpu_fan_percent);
        let gpu_result = fan_control::set_manual_speed(FanId::Gpu, self.gpu_fan_percent);

        self.fan_control_message = match (cpu_result, gpu_result) {
            (Ok(()), Ok(())) => Some(format!(
                "Applied CPU {}% and GPU {}%.",
                self.cpu_fan_percent, self.gpu_fan_percent
            )),
            (Err(cpu_error), Ok(())) => Some(format!("CPU fan update failed: {cpu_error}")),
            (Ok(()), Err(gpu_error)) => Some(format!("GPU fan update failed: {gpu_error}")),
            (Err(cpu_error), Err(gpu_error)) => Some(format!(
                "CPU fan update failed: {cpu_error}; GPU fan update failed: {gpu_error}"
            )),
        };
    }

    fn restore_auto_fan_control(&mut self) {
        self.fan_control_message = match fan_control::set_auto_mode() {
            Ok(()) => Some("Restored automatic fan control.".to_owned()),
            Err(error) => Some(format!("Could not restore automatic fan control: {error}")),
        };
    }
}

fn fan_activity_bar(ui: &mut egui::Ui, label: &str, rpm: Option<u32>) {
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

fn fan_slider_row(ui: &mut egui::Ui, label: &str, percent: &mut u8, rpm: Option<u32>) {
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

fn format_temperature(value: Option<f32>) -> String {
    value
        .map(|temperature| format!("{temperature:.0} C"))
        .unwrap_or_else(|| "Unavailable".to_owned())
}

fn format_rpm(value: Option<u32>) -> String {
    value
        .map(|rpm| format!("{rpm} RPM"))
        .unwrap_or_else(|| "Unavailable".to_owned())
}

fn format_voltage(value: Option<f32>) -> String {
    value
        .map(|voltage| format!("{voltage:.2} V"))
        .unwrap_or_else(|| "Unavailable".to_owned())
}

fn fallback_profile_names() -> Vec<String> {
    [
        "low-power",
        "quiet",
        "balanced",
        "balanced-performance",
        "performance",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
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

fn tray_tooltip(data: &SensorData) -> String {
    format!(
        "CPU: {} | Profile: {}",
        format_temperature(data.cpu_package_temp_celsius),
        data.active_power_profile
            .as_deref()
            .unwrap_or("Unavailable")
    )
}

fn load_window_icon() -> Result<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))?.into_rgba8();
    let (width, height) = image.dimensions();

    Ok(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_profile_names_cleanly() {
        assert_eq!(display_profile_name("low-power"), "Low Power");
        assert_eq!(
            display_profile_name("balanced_performance"),
            "Balanced Performance"
        );
    }

    #[test]
    fn builds_tray_tooltip_from_sensor_data() {
        let data = SensorData {
            cpu_package_temp_celsius: Some(72.4),
            active_power_profile: Some("balanced".to_owned()),
            ..SensorData::default()
        };

        assert_eq!(tray_tooltip(&data), "CPU: 72 C | Profile: balanced");
    }
}
