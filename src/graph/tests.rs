use std::collections::VecDeque;
use std::time::{Duration, Instant, UNIX_EPOCH};

use crate::hardware::sensors::SensorData;

use super::axis::{format_temperature_axis_label, format_time_axis_label};
use super::hover::temperature_hover_text;
use super::{clamp_temperature_for_plot, GraphHistory, GraphVisibility};

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
fn hover_text_can_follow_visible_temperature_series() {
    let mut history = GraphHistory::new();
    let now = Instant::now();
    let data = SensorData {
        cpu_package_temp_celsius: Some(62.0),
        nvidia_gpu_temp_celsius: Some(72.4),
        ..SensorData::default()
    };

    history.push(now, &data);

    let visibility = GraphVisibility {
        cpu_temp: false,
        gpu_temp: true,
    };
    let label = temperature_hover_text(&history, 0.0, &visibility).unwrap();

    assert!(label.contains("Time "));
    assert!(!label.contains("CPU "));
    assert!(label.contains("GPU 72.4 C"));
}

#[test]
fn hover_text_is_blank_without_a_nearby_timestamp_sample() {
    let mut history = GraphHistory::new();
    let now = Instant::now();
    let data = SensorData {
        cpu_package_temp_celsius: Some(62.0),
        nvidia_gpu_temp_celsius: Some(72.4),
        ..SensorData::default()
    };

    history.push(now, &data);

    assert!(temperature_hover_text(&history, -10.0, &GraphVisibility::default()).is_none());
}

fn sensor_data_with_cpu_temp(cpu_temp_celsius: f32) -> SensorData {
    SensorData {
        cpu_package_temp_celsius: Some(cpu_temp_celsius),
        ..SensorData::default()
    }
}
