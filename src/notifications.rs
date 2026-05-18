use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AlertCooldown {
    last_sent_at: Option<Instant>,
    duration: Duration,
}

impl AlertCooldown {
    pub fn new(duration: Duration) -> Self {
        Self {
            last_sent_at: None,
            duration,
        }
    }

    pub fn can_send(&self, now: Instant) -> bool {
        self.last_sent_at
            .map(|last_sent_at| now.duration_since(last_sent_at) >= self.duration)
            .unwrap_or(true)
    }

    pub fn mark_sent(&mut self, now: Instant) {
        self.last_sent_at = Some(now);
    }
}
