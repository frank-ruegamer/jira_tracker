#[macro_use]
extern crate rocket;

use rocket::response::status::Conflict;
use rocket::response::Responder;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use rocket::{Request, State};
use serde::{Deserialize, Serialize};

use stopwatch::Stopwatch;

mod instant_serializer;
mod stopwatch;

#[derive(Debug, Clone, Responder)]
struct OccupiedError {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tracker {
    key: String,
    stopwatch: Mutex<Stopwatch>,
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
struct AppData {
    trackers: RwLock<HashMap<String, Tracker>>,
}

impl AppData {
    fn new() -> Self {
        Self {
            trackers: RwLock::new(HashMap::new()),
        }
    }

    fn with<F, R>(&self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Tracker) -> R,
    {
        self.trackers
            .read()
            .unwrap()
            .get(key)
            .map(|tracker| f(tracker))
    }

    fn create_tracker(&self, key: &str) -> Result<(), OccupiedError> {
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

#[get("/<key>")]
fn elapsed(key: &str, app_data: &State<AppData>) -> Option<String> {
    app_data.with(key, |tracker| {
        let stopwatch = tracker.stopwatch.lock().unwrap();
        humantime::format_duration(stopwatch.total_elapsed_seconds()).to_string()
    })
}

#[post("/<key>")]
fn create(key: &str, app_data: &State<AppData>) -> Result<(), Conflict<()>> {
    app_data.create_tracker(key).map_err(|_| Conflict(None))
}

#[post("/<key>/start")]
fn start(key: &str, app_data: &State<AppData>) {
    app_data.with(key, |tracker| {
        let mut stopwatch = tracker.stopwatch.lock().unwrap();
        stopwatch.start();
    });
}

#[post("/<key>/pause")]
fn pause(key: &str, app_data: &State<AppData>) {
    app_data.with(key, |tracker| {
        let mut stopwatch = tracker.stopwatch.lock().unwrap();
        stopwatch.pause();
    });
}

#[rocket::main]
async fn main() {
    let state = AppData::new();
    state.create_tracker("a");
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![elapsed, create, start, pause])
        .launch()
        .await;
}
