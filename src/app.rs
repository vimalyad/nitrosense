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
        apply_nitro_style(context);
        self.refresh_sensor_snapshot();
        self.handle_window_close_request(context);
        self.handle_tray_action(context);

        egui::SidePanel::left("nitro_navigation")
            .resizable(false)
            .exact_width(176.0)
            .frame(egui::Frame::none().fill(sidebar_color()))
            .show(context, |ui| self.show_navigation(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(app_background_color()))
            .show(context, |ui| {
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
        ui.horizontal_centered(|ui| {
            ui.heading(
                egui::RichText::new("NitroSense")
                    .size(30.0)
                    .color(egui::Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("Acer Nitro AN515-58")
                        .strong()
                        .color(egui::Color32::from_rgb(205, 210, 215)),
                );
            });
        });
        ui.add_space(8.0);
        ui.painter().line_segment(
            [
                ui.cursor().min,
                egui::pos2(ui.available_rect_before_wrap().right(), ui.cursor().min.y),
            ],
            egui::Stroke::new(2.0, accent_color()),
        );
    }

    fn show_navigation(&mut self, ui: &mut egui::Ui) {
        ui.add_space(18.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("NITRO")
                    .size(26.0)
                    .strong()
                    .color(accent_color()),
            );
            ui.label(
                egui::RichText::new("AN515-58")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 156, 162)),
            );
        });

        ui.add_space(28.0);
        nav_button(ui, &mut self.active_tab, AppTab::Overview, "Monitoring");
        nav_button(ui, &mut self.active_tab, AppTab::Graph, "Temperature");
        nav_button(ui, &mut self.active_tab, AppTab::FanControl, "Fan Control");

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(
                    self.sensor_data()
                        .active_power_profile
                        .as_deref()
                        .unwrap_or("profile unavailable"),
                )
                .color(egui::Color32::from_rgb(190, 196, 202)),
            );
            ui.label(egui::RichText::new("Current profile").small().weak());
        });
    }

    fn show_status_strip(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                compact_metric(
                    ui,
                    "CPU",
                    format_temperature(self.sensor_data().cpu_package_temp_celsius),
                );
                compact_metric(
                    ui,
                    "GPU",
                    format_temperature(self.sensor_data().nvidia_gpu_temp_celsius),
                );
                compact_metric(ui, "CPU Fan", format_rpm(self.sensor_data().cpu_fan_rpm));
                compact_metric(ui, "GPU Fan", format_rpm(self.sensor_data().gpu_fan_rpm));
                compact_metric(
                    ui,
                    "Profile",
                    self.sensor_data()
                        .active_power_profile
                        .clone()
                        .unwrap_or_else(|| "Unavailable".to_owned()),
                );
            });
        });
    }

    fn show_power_profile(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
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
        panel_frame()
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.label(egui::RichText::new(title).strong().color(accent_color()));
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(value)
                        .size(24.0)
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                ui.label(egui::RichText::new(detail).small().weak());
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
        ui.columns(2, |columns| {
            columns[0].heading("Monitoring");
            columns[0].add_space(8.0);
            self.show_stats(&mut columns[0]);

            columns[1].heading("Cooling");
            columns[1].add_space(8.0);
            fan_dashboard_panel(&mut columns[1], "CPU Fan", self.sensor_data().cpu_fan_rpm);
            columns[1].add_space(10.0);
            fan_dashboard_panel(&mut columns[1], "GPU Fan", self.sensor_data().gpu_fan_rpm);
            columns[1].add_space(10.0);
            panel_frame().show(&mut columns[1], |ui| {
                ui.label(
                    egui::RichText::new("Battery")
                        .strong()
                        .color(accent_color()),
                );
                ui.label(format_voltage(self.sensor_data().battery_voltage));
            });
        });
    }

    fn show_graph_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Temperature");
        ui.add_space(8.0);
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.graph_visibility.cpu_temp, "CPU Temp");
                ui.checkbox(&mut self.graph_visibility.gpu_temp, "GPU Temp");
            });
            ui.add_space(8.0);
            show_graph(ui, &self.graph_history, &self.graph_visibility);
        });
    }

    fn show_fan_control_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fan Control");
        ui.add_space(8.0);

        panel_frame().show(ui, |ui| {
            if let Some(path) = &self.fan_control_status.acer_hwmon_path {
                ui.label(format!("Acer hwmon: {}", path.display()));
            } else {
                ui.colored_label(warning_color(), "Acer hwmon adapter not found.");
            }

            if !self.fan_control_status.can_control() {
                ui.colored_label(
                    warning_color(),
                    "PWM controls are not available for both fans.",
                );
            }

            ui.label(format!(
                "CPU PWM: {} | GPU PWM: {}",
                format_pwm_state(
                    self.fan_control_status.cpu_pwm,
                    self.fan_control_status.cpu_pwm_enable
                ),
                format_pwm_state(
                    self.fan_control_status.gpu_pwm,
                    self.fan_control_status.gpu_pwm_enable
                )
            ));
        });

        ui.add_space(10.0);
        panel_frame().show(ui, |ui| {
            let cpu_fan_rpm = self.sensor_data().cpu_fan_rpm;
            let gpu_fan_rpm = self.sensor_data().gpu_fan_rpm;

            fan_slider_row(ui, "CPU Fan", &mut self.cpu_fan_percent, cpu_fan_rpm);
            ui.add_space(8.0);
            fan_slider_row(ui, "GPU Fan", &mut self.gpu_fan_percent, gpu_fan_rpm);

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let controls_enabled = self.fan_control_status.can_control();

                if ui
                    .add_enabled(controls_enabled, egui::Button::new("Apply"))
                    .clicked()
                {
                    self.apply_manual_fan_speeds();
                }

                if ui
                    .add_enabled(controls_enabled, egui::Button::new("Auto"))
                    .clicked()
                {
                    self.restore_auto_fan_control();
                }

                if ui.button("Refresh Status").clicked() {
                    self.fan_control_status = FanControlStatus::detect();
                    self.fan_control_message = Some("Fan control status refreshed.".to_owned());
                }
            });
        });

        ui.add_space(10.0);
        self.show_power_profile(ui);

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

fn apply_nitro_style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = app_background_color();
    visuals.panel_fill = app_background_color();
    visuals.widgets.active.bg_fill = accent_color();
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(86, 26, 30);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 38, 43);
    visuals.selection.bg_fill = accent_color();
    context.set_visuals(visuals);
}

fn nav_button(ui: &mut egui::Ui, active_tab: &mut AppTab, tab: AppTab, label: &str) {
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

fn compact_metric(ui: &mut egui::Ui, label: &str, value: String) {
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

fn fan_dashboard_panel(ui: &mut egui::Ui, label: &str, rpm: Option<u32>) {
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

fn panel_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(panel_color())
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(52, 56, 62)))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
}

fn app_background_color() -> egui::Color32 {
    egui::Color32::from_rgb(15, 17, 21)
}

fn sidebar_color() -> egui::Color32 {
    egui::Color32::from_rgb(10, 11, 14)
}

fn panel_color() -> egui::Color32 {
    egui::Color32::from_rgb(28, 31, 36)
}

fn accent_color() -> egui::Color32 {
    egui::Color32::from_rgb(226, 31, 42)
}

fn warning_color() -> egui::Color32 {
    egui::Color32::from_rgb(235, 150, 60)
}

fn format_pwm_state(pwm: Option<u8>, enable: Option<u8>) -> String {
    match (pwm, enable) {
        (Some(pwm), Some(enable)) => format!("{pwm}/255, mode {enable}"),
        (Some(pwm), None) => format!("{pwm}/255"),
        (None, Some(enable)) => format!("mode {enable}"),
        (None, None) => "Unavailable".to_owned(),
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

    #[test]
    fn formats_pwm_state() {
        assert_eq!(format_pwm_state(Some(128), Some(1)), "128/255, mode 1");
        assert_eq!(format_pwm_state(None, None), "Unavailable");
    }
}
