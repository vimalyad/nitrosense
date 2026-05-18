use std::io;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanId {
    Cpu,
    Gpu,
}

impl FanId {
    fn nbfc_index(self) -> &'static str {
        match self {
            Self::Cpu => "0",
            Self::Gpu => "1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanControlStatus {
    pub nbfc_available: bool,
    pub service_available: bool,
}

impl FanControlStatus {
    pub fn detect() -> Self {
        Self {
            nbfc_available: command_succeeds("nbfc", &["--version"]),
            service_available: command_succeeds(
                "systemctl",
                &["is-active", "--quiet", "nbfc_service"],
            ),
        }
    }
}

pub fn set_manual_speed(fan: FanId, percent: u8) -> io::Result<()> {
    let percent = percent.min(100).to_string();
    run_nbfc(&["set", "-f", fan.nbfc_index(), "-s", &percent])
}

pub fn set_auto_mode() -> io::Result<()> {
    run_nbfc(&["set", "--auto"])
}

fn run_nbfc(args: &[&str]) -> io::Result<()> {
    let output = Command::new("nbfc").args(args).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(io::Error::new(
            io::ErrorKind::Other,
            if stderr.is_empty() {
                format!("nbfc command failed: nbfc {}", args.join(" "))
            } else {
                stderr
            },
        ))
    }
}

fn command_succeeds(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_fan_ids_to_nbfc_indexes() {
        assert_eq!(FanId::Cpu.nbfc_index(), "0");
        assert_eq!(FanId::Gpu.nbfc_index(), "1");
    }
}
