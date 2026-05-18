use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Default)]
pub struct HwmonDevices {
    pub coretemp: Option<PathBuf>,
    pub nvidia: Option<PathBuf>,
    pub intel_gpu: Option<PathBuf>,
    pub acer: Option<PathBuf>,
    pub nvme: Option<PathBuf>,
}

impl HwmonDevices {
    pub fn discover() -> io::Result<Self> {
        Self::discover_from("/sys/class/hwmon")
    }

    pub fn discover_from(root: impl AsRef<Path>) -> io::Result<Self> {
        let mut devices = Self::default();

        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            let name_path = path.join("name");

            let Ok(name) = fs::read_to_string(name_path) else {
                continue;
            };

            devices.record_adapter(name.trim(), path);
        }

        Ok(devices)
    }

    fn record_adapter(&mut self, name: &str, path: PathBuf) {
        match classify_hwmon_adapter(name) {
            Some(HwmonAdapter::Coretemp) => self.coretemp.get_or_insert(path),
            Some(HwmonAdapter::Nvidia) => self.nvidia.get_or_insert(path),
            Some(HwmonAdapter::IntelGpu) => self.intel_gpu.get_or_insert(path),
            Some(HwmonAdapter::Acer) => self.acer.get_or_insert(path),
            Some(HwmonAdapter::Nvme) => self.nvme.get_or_insert(path),
            None => return,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HwmonAdapter {
    Coretemp,
    Nvidia,
    IntelGpu,
    Acer,
    Nvme,
}

fn classify_hwmon_adapter(name: &str) -> Option<HwmonAdapter> {
    let normalized = name.trim().to_ascii_lowercase();

    if normalized == "coretemp" {
        Some(HwmonAdapter::Coretemp)
    } else if normalized == "nvidia" {
        Some(HwmonAdapter::Nvidia)
    } else if normalized == "i915" {
        Some(HwmonAdapter::IntelGpu)
    } else if normalized.starts_with("acer") {
        Some(HwmonAdapter::Acer)
    } else if normalized == "nvme" {
        Some(HwmonAdapter::Nvme)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_hwmon_adapters() {
        assert_eq!(
            classify_hwmon_adapter("coretemp"),
            Some(HwmonAdapter::Coretemp)
        );
        assert_eq!(classify_hwmon_adapter("nvidia"), Some(HwmonAdapter::Nvidia));
        assert_eq!(classify_hwmon_adapter("i915"), Some(HwmonAdapter::IntelGpu));
        assert_eq!(
            classify_hwmon_adapter("acer-isa-0000"),
            Some(HwmonAdapter::Acer)
        );
        assert_eq!(classify_hwmon_adapter("nvme"), Some(HwmonAdapter::Nvme));
    }

    #[test]
    fn ignores_unknown_hwmon_adapters() {
        assert_eq!(classify_hwmon_adapter("BAT1-acpi-0"), None);
        assert_eq!(classify_hwmon_adapter(""), None);
    }
}
