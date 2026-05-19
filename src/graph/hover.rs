use eframe::egui;

use super::axis::format_wall_clock_time;
use super::{GraphHistory, GraphVisibility, GRAPH_DATA_WINDOW, HOVER_SAMPLE_TOLERANCE};

pub(super) fn temperature_hover_text(
    history: &GraphHistory,
    hover_x: f64,
    visibility: &GraphVisibility,
) -> Option<String> {
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

            let distance_seconds = (seconds_ago - target_seconds_ago).abs();
            if seconds_ago <= GRAPH_DATA_WINDOW.as_secs_f64()
                && distance_seconds <= HOVER_SAMPLE_TOLERANCE.as_secs_f64()
            {
                Some((sample, distance_seconds))
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

    if visibility.cpu_temp {
        if let Some(cpu_temp) = sample.cpu_temp_celsius {
            lines.push(format!("CPU {:.1} C", cpu_temp));
        }
    }

    if visibility.gpu_temp {
        if let Some(gpu_temp) = sample.gpu_temp_celsius {
            lines.push(format!("GPU {:.1} C", gpu_temp));
        }
    }

    if lines.len() == 1 {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub(super) fn show_graph_hover_label(
    context: &egui::Context,
    graph_rect: egui::Rect,
    pointer_pos: egui::Pos2,
    text: String,
) {
    let offset = egui::vec2(12.0, 12.0);
    let position = egui::pos2(
        (pointer_pos.x + offset.x).min(graph_rect.right() - 170.0),
        (pointer_pos.y + offset.y).min(graph_rect.bottom() - 78.0),
    )
    .max(graph_rect.left_top() + egui::vec2(8.0, 8.0));

    egui::Area::new(egui::Id::new("temperature_graph_hover_label"))
        .order(egui::Order::Tooltip)
        .fixed_pos(position)
        .interactable(false)
        .show(context, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_black_alpha(230))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(95, 190, 220),
                ))
                .rounding(egui::Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
                });
        });
}
