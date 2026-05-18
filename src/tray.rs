#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayTemperatureState {
    Normal,
    Warm,
    Hot,
}

pub fn state_for_cpu_temp(cpu_temp_celsius: Option<f32>) -> TrayTemperatureState {
    match cpu_temp_celsius {
        Some(temp) if temp >= 85.0 => TrayTemperatureState::Hot,
        Some(temp) if temp >= 70.0 => TrayTemperatureState::Warm,
        _ => TrayTemperatureState::Normal,
    }
}
