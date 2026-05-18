use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const HWMON_ROOT: &str = "/sys/class/hwmon";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanId {
    Cpu,
    Gpu,
}

impl FanId {
    fn pwm_index(self) -> u8 {
        match self {
            Self::Cpu => 1,
            Self::Gpu => 2,
        }
    }

    fn from_helper_name(value: &str) -> Option<Self> {
        match value {
            "cpu" => Some(Self::Cpu),
            "gpu" => Some(Self::Gpu),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanControlStatus {
    pub acer_hwmon_path: Option<PathBuf>,
    pub cpu_pwm_available: bool,
    pub gpu_pwm_available: bool,
    pub cpu_pwm_enable: Option<u8>,
    pub gpu_pwm_enable: Option<u8>,
    pub cpu_pwm: Option<u8>,
    pub gpu_pwm: Option<u8>,
}

impl FanControlStatus {
    pub fn detect() -> Self {
        detect_from(HWMON_ROOT)
    }

    pub fn can_control(&self) -> bool {
        self.cpu_pwm_available && self.gpu_pwm_available
    }
}

pub fn set_manual_speeds(cpu_percent: u8, gpu_percent: u8) -> io::Result<()> {
    let cpu_percent = cpu_percent.min(100).to_string();
    let gpu_percent = gpu_percent.min(100).to_string();
    run_pkexec_helper(&["set-manual-both", &cpu_percent, &gpu_percent])
}

pub fn set_auto_mode() -> io::Result<()> {
    run_pkexec_helper(&["set-auto"])
}

pub fn authorize_helper() -> io::Result<()> {
    run_pkexec_helper(&["authorize"])
}

pub fn handle_helper_args(args: impl IntoIterator<Item = OsString>) -> io::Result<bool> {
    let mut args = args.into_iter();
    let _program = args.next();

    if args.next().as_deref() != Some(std::ffi::OsStr::new("--fan-helper")) {
        return Ok(false);
    }

    let Some(command) = args.next().and_then(|value| value.into_string().ok()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing fan helper command",
        ));
    };

    match command.as_str() {
        "authorize" => {
            discover_acer_hwmon(HWMON_ROOT)?.ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "acer hwmon adapter not found")
            })?;
        }
        "set-auto" => set_auto_mode_direct()?,
        "set-manual-both" => {
            let Some(cpu_percent) = args
                .next()
                .and_then(|value| value.into_string().ok())
                .and_then(|value| value.parse::<u8>().ok())
            else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected CPU fan percent from 0 to 100",
                ));
            };

            let Some(gpu_percent) = args
                .next()
                .and_then(|value| value.into_string().ok())
                .and_then(|value| value.parse::<u8>().ok())
            else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected GPU fan percent from 0 to 100",
                ));
            };

            set_manual_speeds_direct(cpu_percent, gpu_percent)?;
        }
        "set-manual" => {
            let Some(fan) = args
                .next()
                .and_then(|value| value.into_string().ok())
                .and_then(|value| FanId::from_helper_name(&value))
            else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected fan helper name cpu or gpu",
                ));
            };

            let Some(percent) = args
                .next()
                .and_then(|value| value.into_string().ok())
                .and_then(|value| value.parse::<u8>().ok())
            else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected fan percent from 0 to 100",
                ));
            };

            set_manual_speed_direct(fan, percent)?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown fan helper command: {command}"),
            ));
        }
    }

    Ok(true)
}

fn set_manual_speeds_direct(cpu_percent: u8, gpu_percent: u8) -> io::Result<()> {
    let hwmon_path = discover_acer_hwmon(HWMON_ROOT)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "acer hwmon adapter not found"))?;

    set_manual_speed_at_path(&hwmon_path, FanId::Cpu, cpu_percent)?;
    set_manual_speed_at_path(&hwmon_path, FanId::Gpu, gpu_percent)
}

fn set_manual_speed_direct(fan: FanId, percent: u8) -> io::Result<()> {
    let hwmon_path = discover_acer_hwmon(HWMON_ROOT)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "acer hwmon adapter not found"))?;

    set_manual_speed_at_path(&hwmon_path, fan, percent)
}

fn set_manual_speed_at_path(hwmon_path: &Path, fan: FanId, percent: u8) -> io::Result<()> {
    let index = fan.pwm_index();
    write_direct_hwmon_value(hwmon_path.join(format!("pwm{index}_enable")), 1)?;
    write_direct_hwmon_value(
        hwmon_path.join(format!("pwm{index}")),
        percent_to_pwm(percent),
    )
}

fn set_auto_mode_direct() -> io::Result<()> {
    let hwmon_path = discover_acer_hwmon(HWMON_ROOT)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "acer hwmon adapter not found"))?;

    write_direct_hwmon_value(hwmon_path.join("pwm1_enable"), 2)?;
    write_direct_hwmon_value(hwmon_path.join("pwm2_enable"), 2)
}

fn detect_from(root: impl AsRef<Path>) -> FanControlStatus {
    let acer_hwmon_path = discover_acer_hwmon(root).ok().flatten();

    FanControlStatus {
        cpu_pwm_available: acer_hwmon_path
            .as_deref()
            .map(|path| has_pwm_control(path, FanId::Cpu))
            .unwrap_or(false),
        gpu_pwm_available: acer_hwmon_path
            .as_deref()
            .map(|path| has_pwm_control(path, FanId::Gpu))
            .unwrap_or(false),
        cpu_pwm_enable: acer_hwmon_path
            .as_deref()
            .and_then(|path| read_pwm_enable(path, FanId::Cpu)),
        gpu_pwm_enable: acer_hwmon_path
            .as_deref()
            .and_then(|path| read_pwm_enable(path, FanId::Gpu)),
        cpu_pwm: acer_hwmon_path
            .as_deref()
            .and_then(|path| read_pwm(path, FanId::Cpu)),
        gpu_pwm: acer_hwmon_path
            .as_deref()
            .and_then(|path| read_pwm(path, FanId::Gpu)),
        acer_hwmon_path,
    }
}

fn discover_acer_hwmon(root: impl AsRef<Path>) -> io::Result<Option<PathBuf>> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let Ok(name) = fs::read_to_string(path.join("name")) else {
            continue;
        };

        if name.trim().to_ascii_lowercase().starts_with("acer") {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn has_pwm_control(hwmon_path: &Path, fan: FanId) -> bool {
    let index = fan.pwm_index();

    hwmon_path.join(format!("pwm{index}")).exists()
        && hwmon_path.join(format!("pwm{index}_enable")).exists()
}

fn read_pwm_enable(hwmon_path: &Path, fan: FanId) -> Option<u8> {
    read_u8_file(hwmon_path.join(format!("pwm{}_enable", fan.pwm_index())))
}

fn read_pwm(hwmon_path: &Path, fan: FanId) -> Option<u8> {
    read_u8_file(hwmon_path.join(format!("pwm{}", fan.pwm_index())))
}

fn read_u8_file(path: impl AsRef<Path>) -> Option<u8> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn run_pkexec_helper(args: &[&str]) -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let mut command = Command::new("pkexec");
    command.arg(executable).arg("--fan-helper").args(args);

    let output = command.output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            if stderr.is_empty() {
                "fan helper authorization failed".to_owned()
            } else {
                stderr
            },
        ))
    }
}

fn write_direct_hwmon_value(path: impl AsRef<Path>, value: u8) -> io::Result<()> {
    fs::write(path, format!("{value}\n"))
}

fn percent_to_pwm(percent: u8) -> u8 {
    ((u16::from(percent.min(100)) * 255 + 50) / 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_fan_ids_to_pwm_indexes() {
        assert_eq!(FanId::Cpu.pwm_index(), 1);
        assert_eq!(FanId::Gpu.pwm_index(), 2);
    }

    #[test]
    fn maps_percent_to_pwm_range() {
        assert_eq!(percent_to_pwm(0), 0);
        assert_eq!(percent_to_pwm(50), 128);
        assert_eq!(percent_to_pwm(100), 255);
        assert_eq!(percent_to_pwm(150), 255);
    }

    #[test]
    fn detects_acer_pwm_controls() {
        let root = unique_test_dir("acer-pwm");
        let hwmon = root.join("hwmon5");
        fs::create_dir_all(&hwmon).unwrap();
        fs::write(hwmon.join("name"), "acer\n").unwrap();
        fs::write(hwmon.join("pwm1"), "255\n").unwrap();
        fs::write(hwmon.join("pwm1_enable"), "1\n").unwrap();
        fs::write(hwmon.join("pwm2"), "128\n").unwrap();
        fs::write(hwmon.join("pwm2_enable"), "1\n").unwrap();

        let status = detect_from(&root);

        assert_eq!(status.acer_hwmon_path, Some(hwmon));
        assert!(status.cpu_pwm_available);
        assert!(status.gpu_pwm_available);
        assert_eq!(status.cpu_pwm, Some(255));
        assert_eq!(status.gpu_pwm, Some(128));

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
