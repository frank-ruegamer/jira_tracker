use std::time::{Duration, Instant};

pub struct Stopwatch {
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

    pub fn elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |t| t.elapsed()) + self.duration_sum
    }

    pub fn pause(&mut self) {
        self.duration_sum += self.elapsed();
        self.start_time = None;
    }
}