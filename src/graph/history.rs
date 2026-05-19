use std::collections::VecDeque;
use std::time::{Instant, SystemTime};

use crate::hardware::sensors::SensorData;

use super::GRAPH_CAPACITY;

#[derive(Debug, Clone, Copy)]
pub struct GraphSample {
    pub(super) sampled_at: Instant,
    pub(super) sampled_wall_time: SystemTime,
    pub(super) cpu_temp_celsius: Option<f32>,
    pub(super) gpu_temp_celsius: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct GraphHistory {
    pub(super) samples: VecDeque<GraphSample>,
    pub(super) capacity: usize,
}

impl GraphHistory {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(GRAPH_CAPACITY),
            capacity: GRAPH_CAPACITY,
        }
    }

    pub fn push(&mut self, sampled_at: Instant, data: &SensorData) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }

        self.samples.push_back(GraphSample {
            sampled_at,
            sampled_wall_time: SystemTime::now(),
            cpu_temp_celsius: data.cpu_package_temp_celsius,
            gpu_temp_celsius: data.nvidia_gpu_temp_celsius,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

impl Default for GraphHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GraphVisibility {
    pub cpu_temp: bool,
    pub gpu_temp: bool,
}

impl Default for GraphVisibility {
    fn default() -> Self {
        Self {
            cpu_temp: true,
            gpu_temp: true,
        }
    }
}
