use core::option::Option;
use core::result::Result;
use core::result::Result::{Err, Ok};
use std::collections::HashMap;
use std::ops::{AddAssign, Deref, DerefMut};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::duration_serializer;
use crate::instant_serializer::SerializableInstant;

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

#[derive(Debug, Serialize)]
pub struct TrackerInformation {
    key: String,
    #[serde(with = "duration_serializer")]
    duration: Duration,
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

    fn elapsed_seconds(&self, key: &str) -> Option<Duration> {
        self.elapsed(key)
            .map(|elapsed| Duration::from_secs(elapsed.as_secs()))
    }

    fn list_trackers(&self) -> Vec<TrackerInformation> {
        self.trackers
            .keys()
            .map(|key| TrackerInformation {
                key: key.to_owned(),
                duration: self.elapsed_seconds(key).unwrap(),
            })
            .collect()
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

    fn reading<F: FnOnce(&InnerAppData) -> T, T>(&self, f: F) -> T {
        let AppData(inner) = self;
        f(inner.read().unwrap().deref())
    }

    fn writing<F: FnOnce(&mut InnerAppData) -> T, T>(&self, f: F) -> T {
        let AppData(inner) = self;
        f(inner.write().unwrap().deref_mut())
    }

    pub fn elapsed(&self, key: &str) -> Option<Duration> {
        self.reading(|a| a.elapsed(key))
    }

    pub fn elapsed_seconds(&self, key: &str) -> Option<Duration> {
        self.reading(|a| a.elapsed_seconds(key))
    }

    pub fn list_trackers(&self) -> Vec<TrackerInformation> {
        self.reading(|a| a.list_trackers())
    }

    pub fn start(&self, key: &str) -> Option<()> {
        self.writing(|a| a.start(key))
    }

    pub fn pause(&self) {
        self.writing(|a| a.pause())
    }

    pub fn create_tracker(&self, key: &str) -> Result<(), OccupiedError> {
        self.writing(|a| a.create_tracker(key))
    }
}
