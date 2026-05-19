use std::time::{Duration, SystemTime, UNIX_EPOCH};

use egui_plot::{GridInput, GridMark};

use super::{GRAPH_LABEL_WINDOW, GRAPH_TICK_INTERVAL};

pub(super) fn five_minute_x_grid_marks(input: GridInput) -> Vec<GridMark> {
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

pub(super) fn temperature_y_grid_marks(_input: GridInput) -> Vec<GridMark> {
    [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        .into_iter()
        .map(|value| GridMark {
            value,
            step_size: 20.0,
        })
        .collect()
}

pub(super) fn format_time_axis_label(latest_wall_time: SystemTime, x_value: f64) -> String {
    if x_value < -(GRAPH_LABEL_WINDOW.as_secs_f64()) || x_value > 0.0 {
        return String::new();
    }

    let seconds_ago = (-x_value).max(0.0);
    format_wall_clock_time(latest_wall_time - Duration::from_secs_f64(seconds_ago))
}

pub(super) fn format_temperature_axis_label(value: f64) -> String {
    if value > 100.0 {
        String::new()
    } else {
        format!("{value:.0}")
    }
}

pub(super) fn format_wall_clock_time(time: SystemTime) -> String {
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
