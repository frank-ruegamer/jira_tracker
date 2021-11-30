#[macro_use]
extern crate rocket;

use rocket::response::status::Conflict;
use rocket::State;

use app_data::AppData;

mod app_data;
mod instant_serializer;
mod stopwatch;

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
    let _ = state.create_tracker("a");
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![elapsed, create, start, pause])
        .launch()
        .await;
}
