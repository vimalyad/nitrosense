pub(crate) mod formatting;
mod views;

use anyhow::{anyhow, Result};
use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::watch;

use crate::app::formatting::{display_profile_name, fallback_profile_names, tray_tooltip};
use crate::config::AppConfig;
use crate::graph::{GraphHistory, GraphVisibility};
use crate::hardware::fan_control::{self, FanControlStatus, FanId};
use crate::hardware::profile::{self, PowerProfile};
use crate::hardware::sensors::SensorData;
use crate::services::notifications::{send_desktop_notification, ThermalAlertState};
use crate::services::polling::{spawn_sensor_polling, SensorSnapshot};
use crate::services::tray::{state_for_cpu_temp, TrayAction, TrayController};
use crate::ui::theme::{app_background_color, apply_nitro_style, sidebar_color};

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
pub(crate) enum AppTab {
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
        apply_nitro_style(context);
        self.refresh_sensor_snapshot();
        self.handle_window_close_request(context);
        self.handle_tray_action(context);

        egui::SidePanel::left("nitro_navigation")
            .resizable(false)
            .exact_width(176.0)
            .frame(egui::Frame::none().fill(sidebar_color()))
            .show(context, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    self.show_navigation(ui);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(app_background_color()))
            .show(context, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.add_space(14.0);
                            self.show_header(ui);
                            ui.add_space(14.0);
                            self.show_status_strip(ui);
                            ui.add_space(14.0);
                            self.show_polling_status(ui);
                            self.show_notification_status(ui);
                            self.show_tray_status(ui);
                            ui.add_space(8.0);
                            self.show_active_tab(ui);
                        });
                    });
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

fn load_window_icon() -> Result<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../../assets/icon.png"))?.into_rgba8();
    let (width, height) = image.dimensions();

    Ok(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}
