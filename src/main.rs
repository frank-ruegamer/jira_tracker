#[macro_use]
extern crate rocket;

use std::sync::{Mutex, RwLock};

use rocket::State;
use serde::{Deserialize, Serialize};

use stopwatch::Stopwatch;

mod instant_serializer;
mod stopwatch;

#[derive(Debug, Serialize, Deserialize)]
struct Tracker {
    key: String,
    stopwatch: Mutex<Stopwatch>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppData {
    trackers: RwLock<Vec<Tracker>>,
}

impl AppData {
    fn with<F, R>(&self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Tracker) -> R,
    {
        let guard = self.trackers.read().unwrap();
        let tracker = guard
            .iter()
            .filter(|t| t.key == key)
            .collect::<Vec<&Tracker>>();

        if tracker.len() > 1 {
            panic!("Found more than one tracker for key {}", key);
        }

        if tracker.is_empty() {
            return None;
        }

        let tracker = *tracker.first().unwrap();
        Some(f(tracker))
    }

    fn create_tracker(&self, key: &str) {
        self.trackers.write().unwrap().push(Tracker {
            key: key.to_string(),
            stopwatch: Mutex::new(Stopwatch::new_and_start()),
        });
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
fn create(key: &str, app_data: &State<AppData>) {
    app_data.create_tracker(key);
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
    let state = AppData {
        trackers: RwLock::new(vec![]),
    };
    state.create_tracker("a");
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![elapsed, create, start, pause])
        .launch()
        .await;
}
