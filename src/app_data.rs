use crate::stopwatch::Stopwatch;
use core::option::Option;
use core::result::Result;
use core::result::Result::{Err, Ok};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Responder)]
pub struct OccupiedError {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tracker {
    pub key: String,
    pub stopwatch: Mutex<Stopwatch>,
}

impl Tracker {
    fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            stopwatch: Mutex::new(Stopwatch::new_and_start()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    trackers: RwLock<HashMap<String, Tracker>>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            trackers: RwLock::new(HashMap::new()),
        }
    }

    pub fn with<F, R>(&self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Tracker) -> R,
    {
        self.trackers
            .read()
            .unwrap()
            .get(key)
            .map(|tracker| f(tracker))
    }

    pub fn create_tracker(&self, key: &str) -> Result<(), OccupiedError> {
        let mut map = self.trackers.write().unwrap();
        if map.contains_key(key) {
            return Err(OccupiedError {
                key: key.to_string(),
            });
        }
        map.insert(key.to_string(), Tracker::new(key));
        Ok(())
    }
}
