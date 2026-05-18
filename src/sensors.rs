use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const PLATFORM_PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";
const BATTERY_ROOT: &str = "/sys/class/power_supply/BAT1";

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

    pub fn read_sensor_data(&self) -> SensorData {
        SensorData {
            cpu_package_temp_celsius: self
                .coretemp
                .as_deref()
                .and_then(read_cpu_package_temp_celsius),
            nvidia_gpu_temp_celsius: self
                .nvidia
                .as_deref()
                .and_then(|path| read_temp_celsius(path, 1)),
            intel_gpu_temp_celsius: self
                .intel_gpu
                .as_deref()
                .and_then(|path| read_temp_celsius(path, 1)),
            cpu_fan_rpm: self.acer.as_deref().and_then(|path| read_fan_rpm(path, 1)),
            gpu_fan_rpm: self.acer.as_deref().and_then(|path| read_fan_rpm(path, 2)),
            nvme_temp_celsius: self
                .nvme
                .as_deref()
                .and_then(|path| read_temp_celsius(path, 1)),
            battery_voltage: read_battery_voltage(),
            active_power_profile: read_trimmed_file(PLATFORM_PROFILE_PATH),
        }
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

pub fn read_current_sensor_data() -> SensorData {
    read_current_sensor_data_result().unwrap_or_default()
}

pub fn read_current_sensor_data_result() -> io::Result<SensorData> {
    HwmonDevices::discover().map(|devices| devices.read_sensor_data())
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

fn read_cpu_package_temp_celsius(hwmon_path: &Path) -> Option<f32> {
    find_temp_input_by_label(hwmon_path, "package id 0")
        .or_else(|| read_temp_celsius(hwmon_path, 1))
}

fn find_temp_input_by_label(hwmon_path: &Path, expected_label: &str) -> Option<f32> {
    let expected_label = expected_label.to_ascii_lowercase();

    for index in 1..=32 {
        let label_path = hwmon_path.join(format!("temp{index}_label"));
        let Some(label) = read_trimmed_file(label_path) else {
            continue;
        };

        if label.to_ascii_lowercase() == expected_label {
            return read_temp_celsius(hwmon_path, index);
        }
    }

    None
}

fn read_temp_celsius(hwmon_path: &Path, index: u8) -> Option<f32> {
    let path = hwmon_path.join(format!("temp{index}_input"));
    read_i64_file(path).map(|millidegrees| millidegrees as f32 / 1000.0)
}

fn read_fan_rpm(hwmon_path: &Path, index: u8) -> Option<u32> {
    let path = hwmon_path.join(format!("fan{index}_input"));
    read_u32_file(path)
}

fn read_battery_voltage() -> Option<f32> {
    read_battery_voltage_from(BATTERY_ROOT)
}

fn read_battery_voltage_from(root: impl AsRef<Path>) -> Option<f32> {
    let root = root.as_ref();

    read_i64_file(root.join("voltage_now"))
        .map(|microvolts| microvolts as f32 / 1_000_000.0)
        .or_else(|| {
            read_i64_file(root.join("voltage_avg"))
                .map(|microvolts| microvolts as f32 / 1_000_000.0)
        })
}

fn read_i64_file(path: impl AsRef<Path>) -> Option<i64> {
    read_trimmed_file(path)?.parse().ok()
}

fn read_u32_file(path: impl AsRef<Path>) -> Option<u32> {
    read_trimmed_file(path)?.parse().ok()
}

fn read_trimmed_file(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
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

    #[test]
    fn converts_millidegree_temperature_to_celsius() {
        let root = unique_test_dir("temp");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("temp1_input"), "68500\n").unwrap();

        assert_eq!(read_temp_celsius(&root, 1), Some(68.5));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reads_cpu_package_temperature_by_label() {
        let root = unique_test_dir("package-temp");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("temp1_label"), "Core 0\n").unwrap();
        fs::write(root.join("temp1_input"), "61000\n").unwrap();
        fs::write(root.join("temp2_label"), "Package id 0\n").unwrap();
        fs::write(root.join("temp2_input"), "72000\n").unwrap();

        assert_eq!(read_cpu_package_temp_celsius(&root), Some(72.0));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn converts_microvolt_battery_voltage_to_volts() {
        let root = unique_test_dir("battery-voltage");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("voltage_now"), "16440000\n").unwrap();

        assert_eq!(read_battery_voltage_from(&root), Some(16.44));

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let unique = format!(
            "nitrosense-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        std::env::temp_dir().join(unique)
    }
}
