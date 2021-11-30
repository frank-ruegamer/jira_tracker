#[macro_use]
extern crate rocket;

use std::sync::Mutex;

use rocket::State;
use serde::{Deserialize, Serialize};

use stopwatch::Stopwatch;

mod instant_serializer;
mod stopwatch;

#[derive(Debug, Serialize, Deserialize)]
struct JiraTracker {
    key: String,
    stopwatch: Mutex<Stopwatch>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppData {
    trackers: Vec<JiraTracker>,
}

impl AppData {
    fn find<'t>(self: &'t AppData, key: &str) -> Option<&'t JiraTracker> {
        self.trackers
            .iter()
            .filter(|t| t.key == key)
            .fold(None, |acc, t| {
                if let Some(_) = acc {
                    panic!("Found more than one tracker for key {}", key)
                }
                Some(t)
            })
    }
}

#[get("/<key>")]
fn elapsed(key: &str, app_data: &State<AppData>) -> Option<String> {
    app_data.find(key).map(|tracker| {
        let stopwatch = tracker.stopwatch.lock().unwrap();
        humantime::format_duration(stopwatch.total_elapsed_seconds()).to_string()
    })
}

#[post("/<key>/start")]
fn start(key: &str, app_data: &State<AppData>) {
    app_data.find(key).map(|tracker| {
        let mut stopwatch = tracker.stopwatch.lock().unwrap();
        stopwatch.start();
    });
}

#[post("/<key>/pause")]
fn pause(key: &str, app_data: &State<AppData>) {
    app_data.find(key).map(|tracker| {
        let mut stopwatch = tracker.stopwatch.lock().unwrap();
        stopwatch.pause();
    });
}

#[rocket::main]
async fn main() {
    let state = AppData {
        trackers: vec![JiraTracker {
            key: "a".to_string(),
            stopwatch: Mutex::new(Stopwatch::new_and_start()),
        }],
    };
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![elapsed, start, pause])
        .launch()
        .await;
}
