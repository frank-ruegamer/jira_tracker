use crate::instant_serializer::SerializableInstant;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::time::{Duration, Instant};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Stopwatch {
    #[serde(default)]
    #[serde_as(as = "Option<SerializableInstant>")]
    start_time: Option<Instant>,
    duration_sum: Duration,
}

impl Stopwatch {
    pub fn new() -> Self {
        Stopwatch {
            start_time: None,
            duration_sum: Duration::ZERO,
        }
    }

    pub fn new_and_start() -> Self {
        let mut stopwatch = Self::new();
        stopwatch.start();
        stopwatch
    }

    pub fn start(&mut self) {
        self.start_time = Option::from(Instant::now());
    }

    fn elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |t| t.elapsed())
    }

    pub fn total_elapsed(&self) -> Duration {
        self.elapsed() + self.duration_sum
    }

    pub fn total_elapsed_seconds(&self) -> Duration {
        Duration::from_secs(self.total_elapsed().as_secs())
    }

    pub fn pause(&mut self) {
        self.duration_sum += self.elapsed();
        self.start_time = None;
    }
}
