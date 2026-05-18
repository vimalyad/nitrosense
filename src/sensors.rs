#[derive(Debug, Clone, Default)]
pub struct SensorData {
    pub cpu_package_temp_celsius: Option<f32>,
    pub nvidia_gpu_temp_celsius: Option<f32>,
    pub intel_gpu_temp_celsius: Option<f32>,
    pub cpu_fan_rpm: Option<u32>,
    pub gpu_fan_rpm: Option<u32>,
    pub nvme_temp_celsius: Option<f32>,
    pub battery_voltage: Option<f32>,
    pub active_power_profile: Option<String>,
}
