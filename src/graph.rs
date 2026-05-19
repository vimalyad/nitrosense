use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;
use egui_plot::{
    CoordinatesFormatter, Corner, GridInput, GridMark, Legend, Line, Plot, PlotBounds, PlotPoints,
};

use crate::hardware::sensors::SensorData;

const GRAPH_DATA_WINDOW: Duration = Duration::from_secs(35 * 60);
const GRAPH_LABEL_WINDOW: Duration = Duration::from_secs(30 * 60);
const GRAPH_TICK_INTERVAL: Duration = Duration::from_secs(5 * 60);
const GRAPH_CAPACITY: usize = 35 * 60;
const TEMPERATURE_MIN_CELSIUS: f64 = 0.0;
const TEMPERATURE_MAX_CELSIUS: f64 = 105.0;

#[derive(Debug, Clone, Copy)]
struct GraphSample {
    sampled_at: Instant,
    sampled_wall_time: SystemTime,
    cpu_temp_celsius: Option<f32>,
    gpu_temp_celsius: Option<f32>,
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
            sampled_wall_time: SystemTime::now(),
            cpu_temp_celsius: data.cpu_package_temp_celsius,
            gpu_temp_celsius: data.nvidia_gpu_temp_celsius,
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
}

impl Default for GraphVisibility {
    fn default() -> Self {
        Self {
            cpu_temp: true,
            gpu_temp: true,
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
        .unwrap_or_else(SystemTime::now);

    let response = Plot::new("temperature_graph")
        .legend(Legend::default())
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
        .label_formatter(move |name, point| {
            format_hover_label(name, latest_wall_time, point.x, point.y)
        })
        .auto_bounds(egui::Vec2b::new(true, false))
        .allow_drag(egui::Vec2b::new(true, false))
        .allow_scroll(egui::Vec2b::new(true, false))
        .allow_zoom(egui::Vec2b::new(true, false))
        .coordinates_formatter(
            Corner::LeftBottom,
            CoordinatesFormatter::new(move |point, _bounds| {
                format!(
                    "time = {}\ntemp = {:.1} C",
                    format_time_for_x_value(latest_wall_time, point.x),
                    point.y
                )
            }),
        )
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                [-(GRAPH_DATA_WINDOW.as_secs_f64()), TEMPERATURE_MIN_CELSIUS],
                [0.0, TEMPERATURE_MAX_CELSIUS],
            ));

            if visibility.cpu_temp {
                add_line(
                    plot_ui,
                    "CPU",
                    history,
                    egui::Color32::from_rgb(70, 170, 95),
                    |sample| sample.cpu_temp_celsius.map(f64::from),
                );
            }

            if visibility.gpu_temp {
                add_line(
                    plot_ui,
                    "GPU",
                    history,
                    egui::Color32::from_rgb(70, 140, 210),
                    |sample| sample.gpu_temp_celsius.map(f64::from),
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

    plot_ui.line(Line::new(PlotPoints::from(points)).name(name).color(color));
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

            if seconds_ago <= GRAPH_DATA_WINDOW.as_secs_f64() {
                Some((sample, (seconds_ago - target_seconds_ago).abs()))
            } else {
                None
            }
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(sample, _)| sample)?;

    let mut lines = Vec::new();

    lines.push(format!(
        "Time {}",
        format_wall_clock_time(sample.sampled_wall_time)
    ));

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

fn five_minute_x_grid_marks(input: GridInput) -> Vec<GridMark> {
    let step = GRAPH_TICK_INTERVAL.as_secs_f64();
    let first = (input.bounds.0 / step).ceil() as i64;
    let last = (input.bounds.1 / step).floor() as i64;

    (first..=last)
        .map(|index| GridMark {
            value: index as f64 * step,
            step_size: step,
        })
        .collect()
}

fn temperature_y_grid_marks(_input: GridInput) -> Vec<GridMark> {
    [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        .into_iter()
        .map(|value| GridMark {
            value,
            step_size: 20.0,
        })
        .collect()
}

fn format_time_axis_label(latest_wall_time: SystemTime, x_value: f64) -> String {
    if x_value < -(GRAPH_LABEL_WINDOW.as_secs_f64()) || x_value > 0.0 {
        return String::new();
    }

    let seconds_ago = (-x_value).max(0.0);
    format_wall_clock_time(latest_wall_time - Duration::from_secs_f64(seconds_ago))
}

fn format_temperature_axis_label(value: f64) -> String {
    if value > 100.0 {
        String::new()
    } else {
        format!("{value:.0}")
    }
}

fn format_hover_label(
    name: &str,
    latest_wall_time: SystemTime,
    x_value: f64,
    y_value: f64,
) -> String {
    let time = format_time_for_x_value(latest_wall_time, x_value);
    let details = format!("time = {time}\ntemp = {y_value:.1} C");

    if name.is_empty() {
        details
    } else {
        format!("{name}\n{details}")
    }
}

fn format_time_for_x_value(latest_wall_time: SystemTime, x_value: f64) -> String {
    let seconds_ago = (-x_value).max(0.0);
    format_wall_clock_time(latest_wall_time - Duration::from_secs_f64(seconds_ago))
}

fn format_wall_clock_time(time: SystemTime) -> String {
    #[cfg(unix)]
    {
        format_local_wall_clock_time(time)
    }

    #[cfg(not(unix))]
    {
        format_utc_wall_clock_time(time)
    }
}

#[cfg(unix)]
fn format_local_wall_clock_time(time: SystemTime) -> String {
    use std::mem::MaybeUninit;
    use std::os::raw::{c_int, c_long};

    #[repr(C)]
    struct Tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        tm_gmtoff: c_long,
        tm_zone: *const std::os::raw::c_char,
    }

    extern "C" {
        fn localtime_r(timep: *const c_long, result: *mut Tm) -> *mut Tm;
    }

    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "--:--".to_owned();
    };

    let timestamp = duration.as_secs() as c_long;
    let mut local_time = MaybeUninit::<Tm>::uninit();

    // The graph only needs HH:MM labels; localtime_r keeps that conversion in the OS timezone.
    let converted = unsafe { localtime_r(&timestamp, local_time.as_mut_ptr()) };
    if converted.is_null() {
        return format_utc_wall_clock_time(time);
    }

    let local_time = unsafe { local_time.assume_init() };
    format!("{:02}:{:02}", local_time.tm_hour, local_time.tm_min)
}

fn format_utc_wall_clock_time(time: SystemTime) -> String {
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "--:--".to_owned();
    };

    let seconds_since_midnight = duration.as_secs() % (24 * 60 * 60);
    let hour = seconds_since_midnight / (60 * 60);
    let minute = (seconds_since_midnight % (60 * 60)) / 60;

    format!("{hour:02}:{minute:02}")
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
        assert_eq!(clamp_temperature_for_plot(108.0), 105.0);
    }

    #[test]
    fn hides_temperature_axis_labels_above_one_hundred() {
        assert_eq!(format_temperature_axis_label(100.0), "100");
        assert_eq!(format_temperature_axis_label(105.0), "");
    }

    #[test]
    fn hides_time_axis_labels_outside_visible_label_window() {
        let now = UNIX_EPOCH + Duration::from_secs(12 * 60 * 60);

        assert!(!format_time_axis_label(now, -(30.0 * 60.0)).is_empty());
        assert_eq!(format_time_axis_label(now, -(35.0 * 60.0)), "");
    }

    #[test]
    fn formats_curve_hover_label_with_named_series_and_units() {
        let now = UNIX_EPOCH + Duration::from_secs(12 * 60 * 60);
        let label = format_hover_label("GPU", now, -60.0, 72.4);

        assert!(label.starts_with("GPU\ntime = "));
        assert!(label.ends_with("\ntemp = 72.4 C"));
    }

    fn sensor_data_with_cpu_temp(cpu_temp_celsius: f32) -> SensorData {
        SensorData {
            cpu_package_temp_celsius: Some(cpu_temp_celsius),
            ..SensorData::default()
        }
    }
}
