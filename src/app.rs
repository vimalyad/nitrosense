use anyhow::{anyhow, Result};
use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::watch;

use crate::graph::{show_graph, GraphHistory, GraphVisibility};
use crate::polling::{spawn_sensor_polling, SensorSnapshot};
use crate::sensors::SensorData;

pub fn run() -> Result<()> {
    let runtime = Runtime::new()?;
    let sensor_receiver = spawn_sensor_polling(runtime.handle());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([920.0, 640.0]),
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

        Self {
            _runtime: runtime,
            sensor_receiver,
            sensor_snapshot,
            graph_history,
            graph_visibility: GraphVisibility::default(),
            active_tab: AppTab::Overview,
        }
    }
}

impl eframe::App for NitroSenseApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_sensor_snapshot();

        egui::CentralPanel::default().show(context, |ui| {
            ui.add_space(8.0);
            self.show_header(ui);
            ui.add_space(12.0);
            self.show_power_profile(ui);
            ui.add_space(12.0);
            self.show_stats(ui);
            self.show_polling_status(ui);
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

    fn show_power_profile(&self, ui: &mut egui::Ui) {
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
                for profile in ["Low Power", "Quiet", "Balanced", "Balanced+", "Performance"] {
                    let active = self
                        .sensor_data()
                        .active_power_profile
                        .as_deref()
                        .map(|current| current.eq_ignore_ascii_case(profile))
                        .unwrap_or(false);
                    ui.add_enabled(false, egui::SelectableLabel::new(active, profile));
                }
            });
        });
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
            ui.checkbox(&mut self.graph_visibility.cpu_fan, "CPU Fan");
            ui.checkbox(&mut self.graph_visibility.gpu_fan, "GPU Fan");
        });
        ui.add_space(8.0);
        show_graph(ui, &self.graph_history, &self.graph_visibility);
    }

    fn show_fan_control_tab(&self, ui: &mut egui::Ui) {
        ui.heading("Fan Control");
        ui.add_space(8.0);
        ui.label("Manual fan controls will be enabled in the NBFC fan control phase.");
        ui.add_space(8.0);
        ui.label(format!(
            "CPU Fan: {}",
            format_rpm(self.sensor_data().cpu_fan_rpm)
        ));
        ui.label(format!(
            "GPU Fan: {}",
            format_rpm(self.sensor_data().gpu_fan_rpm)
        ));
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
