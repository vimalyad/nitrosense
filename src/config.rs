#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cpu_warning_celsius: f32,
    pub cpu_critical_celsius: f32,
    pub gpu_warning_celsius: f32,
    pub gpu_critical_celsius: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cpu_warning_celsius: 85.0,
            cpu_critical_celsius: 95.0,
            gpu_warning_celsius: 80.0,
            gpu_critical_celsius: 90.0,
        }
    }
}
