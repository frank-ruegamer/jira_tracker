use core::option::Option;
use core::result::Result;
use core::result::Result::{Err, Ok};
use std::collections::HashMap;
use std::ops::{AddAssign, Deref, DerefMut};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::config::write_state_file;
use crate::serde::{duration_serializer, instant_serializer};

#[derive(Debug, Clone, Responder)]
pub struct OccupiedError {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PausedTracker {
    pub key: String,
    pub duration: Duration,
    pub start_time: DateTime<Local>,
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

#[derive(Debug, Serialize, Deserialize)]
struct RunningTracker {
    key: String,
    #[serde(with = "instant_serializer")]
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

    fn current(&self) -> Option<TrackerInformation> {
        self.running.as_ref().map(|r| TrackerInformation {
            key: r.key.to_owned(),
            duration: self.elapsed_seconds(&r.key).unwrap(),
        })
    }

    fn get_tracker(&self, key: &str) -> Option<TrackerInformation> {
        self.trackers.get(key).map(|_| TrackerInformation {
            key: key.to_owned(),
            duration: self.elapsed_seconds(key).unwrap(),
        })
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
        if let Some(running) = &self.running {
            *self.trackers.get_mut(&running.key).unwrap() += running;
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

    fn remove(&mut self, key: &str) -> Option<PausedTracker> {
        if self.running.as_ref().filter(|t| t.key == key).is_some() {
            self.pause();
        }
        self.trackers.remove(key)
    }

    fn remove_all(&mut self) -> Vec<PausedTracker> {
        self.pause();
        let map: Vec<String> = self.trackers.keys().map(|k| k.to_string()).collect();
        map.iter()
            .map(|key| self.trackers.remove(key).unwrap())
            .collect()
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
        let result = f(inner.write().unwrap().deref_mut());
        write_state_file(self).unwrap();
        result
    }

    pub fn current(&self) -> Option<TrackerInformation> {
        self.reading(|a| a.current())
    }

    pub fn get_tracker(&self, key: &str) -> Option<TrackerInformation> {
        self.reading(|a| a.get_tracker(key))
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

    pub fn remove(&self, key: &str) -> Option<PausedTracker> {
        self.writing(|a| a.remove(key))
    }

    pub fn remove_all(&self) -> Vec<PausedTracker> {
        self.writing(|a| a.remove_all())
    }
}
