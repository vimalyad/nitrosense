use anyhow::{anyhow, Result};
use eframe::egui;

use crate::sensors::{read_current_sensor_data, SensorData};

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([920.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "NitroSense",
        options,
        Box::new(|creation_context| Box::new(NitroSenseApp::new(creation_context))),
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
    sensor_data: SensorData,
    active_tab: AppTab,
}

impl NitroSenseApp {
    fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        Self {
            sensor_data: read_current_sensor_data(),
            active_tab: AppTab::Overview,
        }
    }
}

impl eframe::App for NitroSenseApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("NitroSense");
        });
    }
}
