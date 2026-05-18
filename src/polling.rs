use std::time::Duration;

use tokio::runtime::Handle;
use tokio::sync::watch;

use crate::sensors::{read_current_sensor_data_result, SensorData};

const SENSOR_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Default)]
pub struct SensorSnapshot {
    pub data: SensorData,
    pub last_error: Option<String>,
}

pub fn spawn_sensor_polling(handle: &Handle) -> watch::Receiver<SensorSnapshot> {
    let initial_snapshot = read_sensor_snapshot();
    let (sender, receiver) = watch::channel(initial_snapshot);

    handle.spawn(async move {
        let mut interval = tokio::time::interval(SENSOR_POLL_INTERVAL);

        loop {
            interval.tick().await;

            let snapshot = match tokio::task::spawn_blocking(read_sensor_snapshot).await {
                Ok(snapshot) => snapshot,
                Err(error) => SensorSnapshot {
                    data: SensorData::default(),
                    last_error: Some(format!("sensor polling task failed: {error}")),
                },
            };

            if sender.send(snapshot).is_err() {
                break;
            }
        }
    });

    receiver
}

fn read_sensor_snapshot() -> SensorSnapshot {
    match read_current_sensor_data_result() {
        Ok(data) => SensorSnapshot {
            data,
            last_error: None,
        },
        Err(error) => SensorSnapshot {
            data: SensorData::default(),
            last_error: Some(error.to_string()),
        },
    }
}
