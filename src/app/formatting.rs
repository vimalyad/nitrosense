use crate::hardware::sensors::SensorData;

pub(crate) fn format_pwm_state(pwm: Option<u8>, enable: Option<u8>) -> String {
    match (pwm, enable) {
        (Some(pwm), Some(enable)) => format!("{pwm}/255, mode {enable}"),
        (Some(pwm), None) => format!("{pwm}/255"),
        (None, Some(enable)) => format!("mode {enable}"),
        (None, None) => "Unavailable".to_owned(),
    }
}

pub(crate) fn format_temperature(value: Option<f32>) -> String {
    value
        .map(|temperature| format!("{temperature:.0} C"))
        .unwrap_or_else(|| "Unavailable".to_owned())
}

pub(crate) fn format_rpm(value: Option<u32>) -> String {
    value
        .map(|rpm| format!("{rpm} RPM"))
        .unwrap_or_else(|| "Unavailable".to_owned())
}

pub(crate) fn format_voltage(value: Option<f32>) -> String {
    value
        .map(|voltage| format!("{voltage:.2} V"))
        .unwrap_or_else(|| "Unavailable".to_owned())
}

pub(crate) fn fallback_profile_names() -> Vec<String> {
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

pub(crate) fn display_profile_name(profile_name: &str) -> String {
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

pub(crate) fn tray_tooltip(data: &SensorData) -> String {
    format!(
        "CPU: {} | Profile: {}",
        format_temperature(data.cpu_package_temp_celsius),
        data.active_power_profile
            .as_deref()
            .unwrap_or("Unavailable")
    )
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
