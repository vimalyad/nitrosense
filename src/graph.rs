mod axis;
mod history;
mod hover;

#[cfg(test)]
mod tests;

use std::time::Duration;

use eframe::egui;
use egui_plot::{Corner, Legend, Line, Plot, PlotBounds, PlotPoints};

use self::axis::{
    five_minute_x_grid_marks, format_temperature_axis_label, format_time_axis_label,
    temperature_y_grid_marks,
};
pub use self::history::{GraphHistory, GraphSample, GraphVisibility};
use self::hover::{show_graph_hover_label, temperature_hover_text};

const GRAPH_DATA_WINDOW: Duration = Duration::from_secs(35 * 60);
const GRAPH_LABEL_WINDOW: Duration = Duration::from_secs(30 * 60);
const GRAPH_TICK_INTERVAL: Duration = Duration::from_secs(5 * 60);
const GRAPH_CAPACITY: usize = 35 * 60;
const HOVER_SAMPLE_TOLERANCE: Duration = Duration::from_secs(2);
const TEMPERATURE_MIN_CELSIUS: f64 = 0.0;
const TEMPERATURE_MAX_CELSIUS: f64 = 105.0;

#[derive(Debug, Clone, Copy)]
enum TemperatureSeries {
    Cpu,
    Gpu,
}

impl TemperatureSeries {
    fn id(self) -> egui::Id {
        match self {
            Self::Cpu => egui::Id::new("temperature_graph_cpu"),
            Self::Gpu => egui::Id::new("temperature_graph_gpu"),
        }
    }
}

pub fn show_graph(ui: &mut egui::Ui, history: &GraphHistory, visibility: &GraphVisibility) {
    if history.is_empty() {
        ui.label("Waiting for sensor samples.");
        return;
    }

    if !visibility.cpu_temp && !visibility.gpu_temp {
        ui.label("Select at least one temperature series to show the graph.");
        return;
    }

    let latest_wall_time = history
        .samples
        .back()
        .map(|sample| sample.sampled_wall_time)
        .unwrap_or_else(std::time::SystemTime::now);

    Plot::new("temperature_graph")
        .legend(
            Legend::default()
                .position(Corner::LeftTop)
                .background_alpha(0.9),
        )
        .height(360.0)
        .y_axis_label("Celsius")
        .include_x(-(GRAPH_DATA_WINDOW.as_secs_f64()))
        .include_x(0.0)
        .include_y(TEMPERATURE_MIN_CELSIUS)
        .include_y(TEMPERATURE_MAX_CELSIUS)
        .x_grid_spacer(five_minute_x_grid_marks)
        .y_grid_spacer(temperature_y_grid_marks)
        .x_axis_formatter(move |mark, _max_chars, _range| {
            format_time_axis_label(latest_wall_time, mark.value)
        })
        .y_axis_formatter(|mark, _max_chars, _range| format_temperature_axis_label(mark.value))
        .label_formatter(|_name, _point| String::new())
        .auto_bounds(egui::Vec2b::new(true, false))
        .allow_drag(egui::Vec2b::new(true, false))
        .allow_scroll(egui::Vec2b::new(true, false))
        .allow_zoom(egui::Vec2b::new(true, false))
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                [-(GRAPH_DATA_WINDOW.as_secs_f64()), TEMPERATURE_MIN_CELSIUS],
                [0.0, TEMPERATURE_MAX_CELSIUS],
            ));

            if visibility.cpu_temp {
                add_line(
                    plot_ui,
                    "CPU",
                    TemperatureSeries::Cpu.id(),
                    history,
                    egui::Color32::from_rgb(70, 170, 95),
                    |sample| sample.cpu_temp_celsius.map(f64::from),
                );
            }

            if visibility.gpu_temp {
                add_line(
                    plot_ui,
                    "GPU",
                    TemperatureSeries::Gpu.id(),
                    history,
                    egui::Color32::from_rgb(70, 140, 210),
                    |sample| sample.gpu_temp_celsius.map(f64::from),
                );
            }

            if plot_ui.response().hovered() {
                if let (Some(point), Some(pointer_pos)) = (
                    plot_ui.pointer_coordinate(),
                    plot_ui.ctx().input(|input| input.pointer.latest_pos()),
                ) {
                    if let Some(text) = temperature_hover_text(history, point.x, visibility) {
                        show_graph_hover_label(
                            plot_ui.ctx(),
                            plot_ui.response().rect,
                            pointer_pos,
                            text,
                        );
                    }
                }
            }
        });
}

fn add_line(
    plot_ui: &mut egui_plot::PlotUi,
    name: &str,
    id: egui::Id,
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

            if seconds_ago <= GRAPH_DATA_WINDOW.as_secs_f64() {
                value(sample).map(|reading| [-seconds_ago, clamp_temperature_for_plot(reading)])
            } else {
                None
            }
        })
        .collect();

    if points.is_empty() {
        return;
    }

    plot_ui.line(
        Line::new(PlotPoints::from(points))
            .name(name)
            .id(id)
            .color(color),
    );
}

fn clamp_temperature_for_plot(value: f64) -> f64 {
    value.clamp(TEMPERATURE_MIN_CELSIUS, TEMPERATURE_MAX_CELSIUS)
}
