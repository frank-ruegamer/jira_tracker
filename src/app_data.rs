use crate::instant_serializer::SerializableInstant;
use chrono::{DateTime, Local};
use core::option::Option;
use core::result::Result;
use core::result::Result::{Err, Ok};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::ops::AddAssign;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Responder)]
pub struct OccupiedError {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PausedTracker {
    key: String,
    duration: Duration,
    start_time: DateTime<Local>,
}

impl PausedTracker {
    fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            duration: Duration::default(),
            start_time: Local::now(),
        }
    }
}

impl AddAssign<&RunningTracker> for PausedTracker {
    fn add_assign(&mut self, rhs: &RunningTracker) {
        self.duration += rhs.start_time.elapsed();
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct RunningTracker {
    key: String,
    #[serde_as(as = "SerializableInstant")]
    start_time: Instant,
}

impl RunningTracker {
    fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            start_time: Instant::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InnerAppData {
    running: Option<RunningTracker>,
    trackers: HashMap<String, PausedTracker>,
}

impl InnerAppData {
    fn new() -> Self {
        Self {
            running: None,
            trackers: HashMap::new(),
        }
    }

    fn elapsed(&self, key: &str) -> Option<Duration> {
        self.trackers.get(key).map(|tracker| {
            let running_duration = self
                .running
                .as_ref()
                .filter(|r| r.key == key)
                .map_or(Duration::ZERO, |r| r.start_time.elapsed());
            tracker.duration + running_duration
        })
    }

    fn start(&mut self, key: &str) -> Option<()> {
        if !self.trackers.contains_key(key) {
            return None;
        }
        self.pause();
        self.running = Some(RunningTracker::new(key));
        Some(())
    }

    fn pause(&mut self) {
        if self.running.is_some() {
            let RunningTracker { key, start_time } = self.running.as_ref().unwrap();
            self.trackers.get_mut(key).unwrap().duration += start_time.elapsed();
        }
        self.running = None;
    }

    fn create_tracker(&mut self, key: &str) -> Result<(), OccupiedError> {
        if self.trackers.contains_key(key) {
            return Err(OccupiedError {
                key: key.to_string(),
            });
        }
        self.trackers
            .insert(key.to_string(), PausedTracker::new(key));
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData(RwLock<InnerAppData>);

impl AppData {
    pub fn new() -> Self {
        Self(RwLock::new(InnerAppData::new()))
    }

    pub fn elapsed(&self, key: &str) -> Option<Duration> {
        let AppData(inner) = self;
        inner.read().unwrap().elapsed(key)
    }

    pub fn elapsed_seconds(&self, key: &str) -> Option<Duration> {
        self.elapsed(key)
            .map(|elapsed| Duration::from_secs(elapsed.as_secs()))
    }

    pub fn start(&self, key: &str) -> Option<()> {
        let AppData(inner) = self;
        inner.write().unwrap().start(key)
    }

    pub fn pause(&self) {
        let AppData(inner) = self;
        inner.write().unwrap().pause();
    }

    pub fn create_tracker(&self, key: &str) -> Result<(), OccupiedError> {
        let AppData(inner) = self;
        inner.write().unwrap().create_tracker(key)
    }
}
