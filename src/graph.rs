use std::collections::VecDeque;
use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::sensors::SensorData;

const GRAPH_WINDOW: Duration = Duration::from_secs(30 * 60);
const GRAPH_CAPACITY: usize = 30 * 60;

#[derive(Debug, Clone, Copy)]
struct GraphSample {
    sampled_at: Instant,
    cpu_temp_celsius: Option<f32>,
    gpu_temp_celsius: Option<f32>,
    cpu_fan_rpm: Option<u32>,
    gpu_fan_rpm: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GraphHistory {
    samples: VecDeque<GraphSample>,
    capacity: usize,
}

impl GraphHistory {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(GRAPH_CAPACITY),
            capacity: GRAPH_CAPACITY,
        }
    }

    pub fn push(&mut self, sampled_at: Instant, data: &SensorData) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }

        self.samples.push_back(GraphSample {
            sampled_at,
            cpu_temp_celsius: data.cpu_package_temp_celsius,
            gpu_temp_celsius: data.nvidia_gpu_temp_celsius.or(data.intel_gpu_temp_celsius),
            cpu_fan_rpm: data.cpu_fan_rpm,
            gpu_fan_rpm: data.gpu_fan_rpm,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

impl Default for GraphHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GraphVisibility {
    pub cpu_temp: bool,
    pub gpu_temp: bool,
    pub cpu_fan: bool,
    pub gpu_fan: bool,
}

impl Default for GraphVisibility {
    fn default() -> Self {
        Self {
            cpu_temp: true,
            gpu_temp: true,
            cpu_fan: true,
            gpu_fan: true,
        }
    }
}

pub fn show_graph(ui: &mut egui::Ui, history: &GraphHistory, visibility: &GraphVisibility) {
    if history.is_empty() {
        ui.label("Waiting for sensor samples.");
        return;
    }

    Plot::new("sensor_graph")
        .legend(Legend::default())
        .height(360.0)
        .include_x(-(GRAPH_WINDOW.as_secs_f64()))
        .include_x(0.0)
        .show(ui, |plot_ui| {
            if visibility.cpu_temp {
                add_line(
                    plot_ui,
                    "CPU Temp C",
                    history,
                    egui::Color32::from_rgb(70, 170, 95),
                    |sample| sample.cpu_temp_celsius.map(f64::from),
                );
            }

            if visibility.gpu_temp {
                add_line(
                    plot_ui,
                    "GPU Temp C",
                    history,
                    egui::Color32::from_rgb(70, 140, 210),
                    |sample| sample.gpu_temp_celsius.map(f64::from),
                );
            }

            if visibility.cpu_fan {
                add_line(
                    plot_ui,
                    "CPU Fan RPM",
                    history,
                    egui::Color32::from_rgb(220, 170, 65),
                    |sample| sample.cpu_fan_rpm.map(f64::from),
                );
            }

            if visibility.gpu_fan {
                add_line(
                    plot_ui,
                    "GPU Fan RPM",
                    history,
                    egui::Color32::from_rgb(210, 95, 95),
                    |sample| sample.gpu_fan_rpm.map(f64::from),
                );
            }
        });
}

fn add_line(
    plot_ui: &mut egui_plot::PlotUi,
    name: &str,
    history: &GraphHistory,
    color: egui::Color32,
    value: impl Fn(&GraphSample) -> Option<f64>,
) {
    let Some(latest_sample) = history.samples.back() else {
        return;
    };

    let points: Vec<[f64; 2]> = history
        .samples
        .iter()
        .filter_map(|sample| {
            let seconds_ago = latest_sample
                .sampled_at
                .duration_since(sample.sampled_at)
                .as_secs_f64();

            if seconds_ago <= GRAPH_WINDOW.as_secs_f64() {
                value(sample).map(|reading| [-seconds_ago, reading])
            } else {
                None
            }
        })
        .collect();

    if points.is_empty() {
        return;
    }

    plot_ui.line(Line::new(PlotPoints::from(points)).name(name).color(color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_history_is_capped_to_rolling_window_capacity() {
        let mut history = GraphHistory {
            samples: VecDeque::new(),
            capacity: 2,
        };
        let now = Instant::now();

        history.push(now, &sensor_data_with_cpu_temp(60.0));
        history.push(
            now + Duration::from_secs(1),
            &sensor_data_with_cpu_temp(61.0),
        );
        history.push(
            now + Duration::from_secs(2),
            &sensor_data_with_cpu_temp(62.0),
        );

        assert_eq!(history.samples.len(), 2);
        assert_eq!(
            history.samples.front().unwrap().cpu_temp_celsius,
            Some(61.0)
        );
        assert_eq!(history.samples.back().unwrap().cpu_temp_celsius, Some(62.0));
    }

    fn sensor_data_with_cpu_temp(cpu_temp_celsius: f32) -> SensorData {
        SensorData {
            cpu_package_temp_celsius: Some(cpu_temp_celsius),
            ..SensorData::default()
        }
    }
}
