use std::collections::HashMap;
use std::time::{Duration, Instant};

use notify_rust::Notification;

use crate::config::AppConfig;
use crate::sensors::SensorData;

const WARNING_COOLDOWN: Duration = Duration::from_secs(5 * 60);
const CRITICAL_COOLDOWN: Duration = Duration::from_secs(2 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertKind {
    CpuWarning,
    CpuCritical,
    GpuWarning,
    GpuCritical,
}

impl AlertKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::CpuWarning => "CPU temperature warning",
            Self::CpuCritical => "CPU temperature critical",
            Self::GpuWarning => "GPU temperature warning",
            Self::GpuCritical => "GPU temperature critical",
        }
    }

    fn cooldown(self) -> Duration {
        match self {
            Self::CpuWarning | Self::GpuWarning => WARNING_COOLDOWN,
            Self::CpuCritical | Self::GpuCritical => CRITICAL_COOLDOWN,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermalAlert {
    pub kind: AlertKind,
    pub temperature_celsius: f32,
}

impl ThermalAlert {
    pub fn message(&self) -> String {
        format!("{:.0} C", self.temperature_celsius)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThermalAlertState {
    last_sent_at: HashMap<AlertKind, Instant>,
}

impl ThermalAlertState {
    pub fn pending_alerts(
        &mut self,
        data: &SensorData,
        config: &AppConfig,
        now: Instant,
    ) -> Vec<ThermalAlert> {
        evaluate_alerts(data, config)
            .into_iter()
            .filter(|alert| self.can_send(alert.kind, now))
            .collect()
    }

    pub fn mark_sent(&mut self, alert: &ThermalAlert, now: Instant) {
        self.last_sent_at.insert(alert.kind, now);
    }

    fn can_send(&self, kind: AlertKind, now: Instant) -> bool {
        self.last_sent_at
            .get(&kind)
            .map(|last_sent_at| now.duration_since(*last_sent_at) >= kind.cooldown())
            .unwrap_or(true)
    }
}

pub fn evaluate_alerts(data: &SensorData, config: &AppConfig) -> Vec<ThermalAlert> {
    let mut alerts = Vec::new();

    if let Some(cpu_temp) = data.cpu_package_temp_celsius {
        if cpu_temp >= config.cpu_critical_celsius {
            alerts.push(ThermalAlert {
                kind: AlertKind::CpuCritical,
                temperature_celsius: cpu_temp,
            });
        } else if cpu_temp >= config.cpu_warning_celsius {
            alerts.push(ThermalAlert {
                kind: AlertKind::CpuWarning,
                temperature_celsius: cpu_temp,
            });
        }
    }

    let gpu_temp = data.nvidia_gpu_temp_celsius.or(data.intel_gpu_temp_celsius);

    if let Some(gpu_temp) = gpu_temp {
        if gpu_temp >= config.gpu_critical_celsius {
            alerts.push(ThermalAlert {
                kind: AlertKind::GpuCritical,
                temperature_celsius: gpu_temp,
            });
        } else if gpu_temp >= config.gpu_warning_celsius {
            alerts.push(ThermalAlert {
                kind: AlertKind::GpuWarning,
                temperature_celsius: gpu_temp,
            });
        }
    }

    alerts
}

pub fn send_desktop_notification(alert: &ThermalAlert) -> notify_rust::error::Result<()> {
    Notification::new()
        .summary(alert.kind.title())
        .body(&alert.message())
        .appname("NitroSense")
        .show()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_warning_and_critical_thresholds() {
        let config = AppConfig::default();
        let data = SensorData {
            cpu_package_temp_celsius: Some(96.0),
            nvidia_gpu_temp_celsius: Some(82.0),
            ..SensorData::default()
        };

        let alerts = evaluate_alerts(&data, &config);

        assert_eq!(
            alerts,
            vec![
                ThermalAlert {
                    kind: AlertKind::CpuCritical,
                    temperature_celsius: 96.0,
                },
                ThermalAlert {
                    kind: AlertKind::GpuWarning,
                    temperature_celsius: 82.0,
                },
            ]
        );
    }

    #[test]
    fn suppresses_alerts_during_cooldown() {
        let config = AppConfig::default();
        let data = SensorData {
            cpu_package_temp_celsius: Some(86.0),
            ..SensorData::default()
        };
        let now = Instant::now();
        let mut state = ThermalAlertState::default();

        let first = state.pending_alerts(&data, &config, now);
        assert_eq!(first.len(), 1);
        state.mark_sent(&first[0], now);

        let suppressed = state.pending_alerts(&data, &config, now + Duration::from_secs(60));
        assert!(suppressed.is_empty());

        let allowed = state.pending_alerts(&data, &config, now + WARNING_COOLDOWN);
        assert_eq!(allowed.len(), 1);
    }
}
