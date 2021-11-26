#[macro_use]
extern crate rocket;

use rocket::State;
use std::sync::Mutex;
use stopwatch::Stopwatch;

mod instant_serializer;
mod stopwatch;

struct AppData {
    stopwatch: Mutex<Stopwatch>,
}

#[get("/")]
fn elapsed(app_data: &State<AppData>) -> String {
    let stopwatch = app_data.stopwatch.lock().unwrap();
    humantime::format_duration(stopwatch.total_elapsed_seconds()).to_string()
}

#[post("/start")]
fn start(app_data: &State<AppData>) {
    let mut stopwatch = app_data.stopwatch.lock().unwrap();
    stopwatch.start();
}

#[post("/pause")]
fn pause(app_data: &State<AppData>) {
    let mut stopwatch = app_data.stopwatch.lock().unwrap();
    stopwatch.pause();
}

#[rocket::main]
async fn main() {
    let state = AppData {
        stopwatch: Mutex::new(Stopwatch::new_and_start()),
    };
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![elapsed, start, pause])
        .launch()
        .await;
}
