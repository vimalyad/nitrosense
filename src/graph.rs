use std::collections::VecDeque;
use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{CoordinatesFormatter, Corner, Legend, Line, Plot, PlotBounds, PlotPoints};

use crate::sensors::SensorData;

const GRAPH_WINDOW: Duration = Duration::from_secs(30 * 60);
const GRAPH_CAPACITY: usize = 30 * 60;
const TEMPERATURE_MIN_CELSIUS: f64 = 0.0;
const TEMPERATURE_MAX_CELSIUS: f64 = 100.0;

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

    let show_temperature_plot = visibility.cpu_temp || visibility.gpu_temp;
    let show_fan_plot = visibility.cpu_fan || visibility.gpu_fan;

    if show_temperature_plot {
        ui.strong("Temperature");
        let response = Plot::new("temperature_graph")
            .legend(Legend::default())
            .height(if show_fan_plot { 220.0 } else { 360.0 })
            .include_x(-(GRAPH_WINDOW.as_secs_f64()))
            .include_x(0.0)
            .include_y(TEMPERATURE_MIN_CELSIUS)
            .include_y(TEMPERATURE_MAX_CELSIUS)
            .auto_bounds(egui::Vec2b::new(true, false))
            .allow_drag(egui::Vec2b::new(true, false))
            .allow_scroll(egui::Vec2b::new(true, false))
            .allow_zoom(egui::Vec2b::new(true, false))
            .coordinates_formatter(
                Corner::LeftBottom,
                CoordinatesFormatter::new(|point, _bounds| {
                    format!("{:.0}s ago\n{:.0} C", -point.x, point.y)
                }),
            )
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    [-(GRAPH_WINDOW.as_secs_f64()), TEMPERATURE_MIN_CELSIUS],
                    [0.0, TEMPERATURE_MAX_CELSIUS],
                ));

                if visibility.cpu_temp {
                    add_line(
                        plot_ui,
                        "CPU Temp C",
                        history,
                        egui::Color32::from_rgb(70, 170, 95),
                        |sample| sample.cpu_temp_celsius.map(f64::from),
                        clamp_temperature_for_plot,
                    );
                }

                if visibility.gpu_temp {
                    add_line(
                        plot_ui,
                        "GPU Temp C",
                        history,
                        egui::Color32::from_rgb(70, 140, 210),
                        |sample| sample.gpu_temp_celsius.map(f64::from),
                        clamp_temperature_for_plot,
                    );
                }

                plot_ui.pointer_coordinate().map(|point| point.x)
            });

        if let Some(hover_x) = response.inner {
            if response.response.hovered() {
                if let Some(text) = temperature_hover_text(history, hover_x) {
                    response.response.on_hover_text(text);
                }
            }
        }
    }

    if show_fan_plot {
        if show_temperature_plot {
            ui.add_space(12.0);
        }

        ui.strong("Fan Speed");
        Plot::new("fan_graph")
            .legend(Legend::default())
            .height(if show_temperature_plot { 220.0 } else { 360.0 })
            .include_x(-(GRAPH_WINDOW.as_secs_f64()))
            .include_x(0.0)
            .include_y(0.0)
            .coordinates_formatter(
                Corner::LeftBottom,
                CoordinatesFormatter::new(|point, _bounds| {
                    format!("{:.0}s ago\n{:.0} RPM", -point.x, point.y)
                }),
            )
            .show(ui, |plot_ui| {
                if visibility.cpu_fan {
                    add_line(
                        plot_ui,
                        "CPU Fan RPM",
                        history,
                        egui::Color32::from_rgb(220, 170, 65),
                        |sample| sample.cpu_fan_rpm.map(f64::from),
                        identity_plot_value,
                    );
                }

                if visibility.gpu_fan {
                    add_line(
                        plot_ui,
                        "GPU Fan RPM",
                        history,
                        egui::Color32::from_rgb(210, 95, 95),
                        |sample| sample.gpu_fan_rpm.map(f64::from),
                        identity_plot_value,
                    );
                }
            });
    }

    if !show_temperature_plot && !show_fan_plot {
        ui.label("Select at least one series to show the graph.");
    }
}

fn add_line(
    plot_ui: &mut egui_plot::PlotUi,
    name: &str,
    history: &GraphHistory,
    color: egui::Color32,
    value: impl Fn(&GraphSample) -> Option<f64>,
    plot_value: impl Fn(f64) -> f64,
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
                value(sample).map(|reading| [-seconds_ago, plot_value(reading)])
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

fn identity_plot_value(value: f64) -> f64 {
    value
}

fn clamp_temperature_for_plot(value: f64) -> f64 {
    value.clamp(TEMPERATURE_MIN_CELSIUS, TEMPERATURE_MAX_CELSIUS)
}

fn temperature_hover_text(history: &GraphHistory, hover_x: f64) -> Option<String> {
    let latest_sample = history.samples.back()?;
    let target_seconds_ago = -hover_x;

    let sample = history
        .samples
        .iter()
        .filter_map(|sample| {
            let seconds_ago = latest_sample
                .sampled_at
                .duration_since(sample.sampled_at)
                .as_secs_f64();

            if seconds_ago <= GRAPH_WINDOW.as_secs_f64() {
                Some((sample, (seconds_ago - target_seconds_ago).abs()))
            } else {
                None
            }
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(sample, _)| sample)?;

    let mut lines = Vec::new();

    if let Some(cpu_temp) = sample.cpu_temp_celsius {
        lines.push(format!("CPU {:.1} C", cpu_temp));
    }

    if let Some(gpu_temp) = sample.gpu_temp_celsius {
        lines.push(format!("GPU {:.1} C", gpu_temp));
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
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

    #[test]
    fn temperature_plot_values_are_clamped_to_fixed_axis() {
        assert_eq!(clamp_temperature_for_plot(-10.0), 0.0);
        assert_eq!(clamp_temperature_for_plot(72.5), 72.5);
        assert_eq!(clamp_temperature_for_plot(108.0), 100.0);
    }

    fn sensor_data_with_cpu_temp(cpu_temp_celsius: f32) -> SensorData {
        SensorData {
            cpu_package_temp_celsius: Some(cpu_temp_celsius),
            ..SensorData::default()
        }
    }
}
