use core::option::Option;
use core::result::Result;
use core::result::Result::{Err, Ok};
use std::ops::{AddAssign, Deref, DerefMut};
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Local};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::files;
use crate::serde::instant_serializer;

#[derive(Debug)]
pub enum TrackerError {
    KeyFormatError,
    OccupiedError,
    NotFoundError,
    DurationAdjustmentError,
}

impl IntoResponse for TrackerError {
    fn into_response(self) -> Response {
        let status_code = match self {
            TrackerError::KeyFormatError => StatusCode::BAD_REQUEST,
            TrackerError::OccupiedError => StatusCode::CONFLICT,
            TrackerError::NotFoundError => StatusCode::NOT_FOUND,
            TrackerError::DurationAdjustmentError => StatusCode::BAD_REQUEST,
        };
        status_code.into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PausedTracker {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    duration: Duration,
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    positive_adjustments: Vec<Duration>,
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    negative_adjustments: Vec<Duration>,
    start_time: DateTime<Local>,
}

impl PausedTracker {
    fn new() -> Self {
        Self {
            description: None,
            duration: Duration::default(),
            positive_adjustments: Vec::new(),
            negative_adjustments: Vec::new(),
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
    pub key: String,
    pub description: Option<String>,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub running: bool,
    pub start_time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InnerAppData {
    running: Option<RunningTracker>,
    trackers: IndexMap<String, PausedTracker>,
}

impl InnerAppData {
    fn new() -> Self {
        Self {
            running: None,
            trackers: IndexMap::new(),
        }
    }

    fn elapsed(&self, key: &str) -> Option<Duration> {
        self.trackers.get(key).map(|tracker| {
            let running_duration = self
                .running
                .as_ref()
                .filter(|r| r.key == key)
                .map_or(Duration::ZERO, |r| r.start_time.elapsed());
            let positive_adjustments_sum: Duration = tracker.positive_adjustments.iter().sum();
            let negative_adjustments_sum: Duration = tracker.negative_adjustments.iter().sum();
            let positive_duration_sum =
                tracker.duration + running_duration + positive_adjustments_sum;
            positive_duration_sum.saturating_sub(negative_adjustments_sum)
        })
    }

    fn elapsed_seconds(&self, key: &str) -> Option<Duration> {
        self.elapsed(key)
            .map(|elapsed| Duration::from_secs(elapsed.as_secs()))
    }

    /// It is assumed that a tracker with the key exists
    fn get_information(&self, key: &str) -> TrackerInformation {
        let tracker = self.trackers.get(key).unwrap();
        TrackerInformation {
            key: key.to_owned(),
            description: tracker.description.clone(),
            duration: self.elapsed_seconds(key).unwrap(),
            running: self
                .running
                .as_ref()
                .filter(|running| running.key == key)
                .is_some(),
            start_time: tracker.start_time,
        }
    }

    fn current(&self) -> Result<TrackerInformation, TrackerError> {
        self.running
            .as_ref()
            .map(|running| self.get_information(&running.key))
            .ok_or(TrackerError::NotFoundError)
    }

    fn get_tracker(&self, key: &str) -> Result<TrackerInformation, TrackerError> {
        self.trackers
            .get(key)
            .map(|_| self.get_information(key))
            .ok_or(TrackerError::NotFoundError)
    }

    fn list_trackers(&self) -> Vec<TrackerInformation> {
        self.trackers
            .keys()
            .map(|key| self.get_information(key))
            .collect()
    }

    fn set_description(
        &mut self,
        key: &str,
        description: Option<String>,
    ) -> Result<TrackerInformation, TrackerError> {
        let description = description.filter(|d| !d.is_empty());
        self.trackers
            .get_mut(key)
            .map(|tracker| tracker.description = description)
            .ok_or(TrackerError::NotFoundError)?;
        Ok(self.get_information(key))
    }

    fn adjust_positive_duration(
        &mut self,
        key: &str,
        duration: Duration,
    ) -> Result<TrackerInformation, TrackerError> {
        self.trackers
            .get_mut(key)
            .map(|tracker| tracker.positive_adjustments.push(duration))
            .ok_or(TrackerError::NotFoundError)?;
        Ok(self.get_information(key))
    }

    fn adjust_negative_duration(
        &mut self,
        key: &str,
        duration: Duration,
    ) -> Result<TrackerInformation, TrackerError> {
        let elapsed = self.elapsed(key).ok_or(TrackerError::NotFoundError)?;
        if duration > elapsed {
            return Err(TrackerError::DurationAdjustmentError);
        }

        self.trackers
            .get_mut(key)
            .unwrap()
            .negative_adjustments
            .push(duration);
        Ok(self.get_information(key))
    }

    fn start(&mut self, key: &str) -> Result<TrackerInformation, TrackerError> {
        if !self.trackers.contains_key(key) {
            return Err(TrackerError::NotFoundError);
        }
        self.pause();
        self.running = Some(RunningTracker::new(key));
        Ok(self.get_information(key))
    }

    fn pause(&mut self) {
        if let Some(running) = &self.running {
            *self.trackers.get_mut(&running.key).unwrap() += running;
        }
        self.running = None;
    }

    fn create_tracker(&mut self, key: &str) -> Result<TrackerInformation, TrackerError> {
        if !Regex::new(r"\w+-\d+").unwrap().is_match(key) {
            return Err(TrackerError::KeyFormatError);
        }
        if self.trackers.contains_key(key) {
            return Err(TrackerError::OccupiedError);
        }
        self.trackers.insert(key.to_string(), PausedTracker::new());
        Ok(self.get_information(key))
    }

    fn remove(&mut self, key: &str) -> Result<PausedTracker, TrackerError> {
        if self.running.as_ref().filter(|t| t.key == key).is_some() {
            self.pause();
        }
        self.trackers
            .shift_remove(key)
            .ok_or(TrackerError::NotFoundError)
    }

    fn remove_all(&mut self) -> Vec<PausedTracker> {
        self.pause();
        let map: Vec<String> = self.trackers.keys().map(|k| k.to_string()).collect();
        map.iter()
            .map(|key| self.trackers.remove(key).unwrap())
            .collect()
    }

    fn sum(&self) -> Duration {
        self.list_trackers().into_iter().map(|t| t.duration).sum()
    }
}

#[derive(Debug)]
pub struct AppData {
    inner: RwLock<InnerAppData>,
    path: PathBuf,
}

impl AppData {
    fn reading<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&InnerAppData) -> T,
    {
        let AppData { inner, .. } = self;
        f(inner.read().unwrap().deref())
    }

    fn writing<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut InnerAppData) -> T,
    {
        let result = self.writing_without_flush(f);
        files::write_file(&self.path, &self.inner).unwrap();
        result
    }

    fn writing_without_flush<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut InnerAppData) -> T,
    {
        let AppData { inner, .. } = self;
        f(inner.write().unwrap().deref_mut())
    }

    pub fn current(&self) -> Result<TrackerInformation, TrackerError> {
        self.reading(|a| a.current())
    }

    pub fn get_tracker(&self, key: &str) -> Result<TrackerInformation, TrackerError> {
        self.reading(|a| a.get_tracker(key))
    }

    pub fn list_trackers(&self) -> Vec<TrackerInformation> {
        self.reading(|a| a.list_trackers())
    }

    pub fn set_description(
        &self,
        key: &str,
        description: Option<String>,
    ) -> Result<TrackerInformation, TrackerError> {
        self.writing(|a| a.set_description(key, description))
    }

    pub fn adjust_positive_duration(
        &self,
        key: &str,
        duration: Duration,
    ) -> Result<TrackerInformation, TrackerError> {
        self.writing(|a| a.adjust_positive_duration(key, duration))
    }

    pub fn adjust_negative_duration(
        &self,
        key: &str,
        duration: Duration,
    ) -> Result<TrackerInformation, TrackerError> {
        self.writing(|a| a.adjust_negative_duration(key, duration))
    }

    pub fn start(&self, key: &str) -> Result<TrackerInformation, TrackerError> {
        self.writing(|a| a.start(key))
    }

    pub fn pause(&self) {
        self.writing(|a| a.pause())
    }

    pub fn create_tracker(&self, key: &str) -> Result<TrackerInformation, TrackerError> {
        self.writing(|a| a.create_tracker(key))
    }

    pub fn remove(&self, key: &str) -> Result<PausedTracker, TrackerError> {
        self.writing(|a| a.remove(key))
    }

    pub fn remove_all(&self) -> Vec<PausedTracker> {
        self.writing(|a| a.remove_all())
    }

    pub fn sum(&self) -> Duration {
        self.reading(|a| a.sum())
    }

    pub fn reload_state(&self) {
        self.writing_without_flush(|a| *a = files::read_file(&self.path).unwrap())
    }
}

impl From<&AppConfig> for AppData {
    fn from(config: &AppConfig) -> Self {
        let path = config.get_state_file();
        let inner = files::read_file(&path).unwrap_or_else(|_| InnerAppData::new());
        AppData {
            inner: RwLock::new(inner),
            path,
        }
    }
}
