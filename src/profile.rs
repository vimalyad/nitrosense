use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

const PLATFORM_PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";
const PLATFORM_PROFILE_CHOICES_PATH: &str = "/sys/firmware/acpi/platform_profile_choices";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerProfile {
    pub name: String,
}

impl PowerProfile {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

pub fn read_active_profile() -> io::Result<Option<PowerProfile>> {
    read_active_profile_from(PLATFORM_PROFILE_PATH)
}

pub fn read_profile_choices() -> io::Result<Vec<PowerProfile>> {
    read_profile_choices_from(PLATFORM_PROFILE_CHOICES_PATH)
}

pub fn set_active_profile(profile: &str) -> io::Result<()> {
    set_active_profile_at(PLATFORM_PROFILE_PATH, profile)
}

fn set_active_profile_at(path: &str, profile: &str) -> io::Result<()> {
    let mut child = Command::new("sudo")
        .arg("-n")
        .arg("tee")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(profile.as_bytes())?;
        stdin.write_all(b"\n")?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            if error.is_empty() {
                "failed to write platform profile with sudo tee".to_owned()
            } else {
                error
            },
        ))
    }
}

fn read_active_profile_from(path: impl AsRef<Path>) -> io::Result<Option<PowerProfile>> {
    Ok(read_trimmed_file(path)?.map(PowerProfile::new))
}

fn read_profile_choices_from(path: impl AsRef<Path>) -> io::Result<Vec<PowerProfile>> {
    let Some(choices) = read_trimmed_file(path)? else {
        return Ok(Vec::new());
    };

    Ok(choices.split_whitespace().map(PowerProfile::new).collect())
}

fn read_trimmed_file(path: impl AsRef<Path>) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(value) => {
            Ok(value.trim().to_owned()).map(
                |value| {
                    if value.is_empty() {
                        None
                    } else {
                        Some(value)
                    }
                },
            )
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_active_profile() {
        let path = unique_test_file("active-profile");
        fs::write(&path, "balanced\n").unwrap();

        let active = read_active_profile_from(&path).unwrap();

        assert_eq!(active, Some(PowerProfile::new("balanced")));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn reads_profile_choices() {
        let path = unique_test_file("profile-choices");
        fs::write(&path, "low-power quiet balanced performance\n").unwrap();

        let choices = read_profile_choices_from(&path).unwrap();

        assert_eq!(
            choices,
            vec![
                PowerProfile::new("low-power"),
                PowerProfile::new("quiet"),
                PowerProfile::new("balanced"),
                PowerProfile::new("performance"),
            ]
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn empty_choices_file_returns_no_profiles() {
        let path = unique_test_file("empty-profile-choices");
        fs::write(&path, "\n").unwrap();

        let choices = read_profile_choices_from(&path).unwrap();

        assert!(choices.is_empty());
        fs::remove_file(path).unwrap();
    }

    fn unique_test_file(name: &str) -> std::path::PathBuf {
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
