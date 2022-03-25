#[macro_use]
extern crate rocket;

use rocket::response::status::Conflict;
use rocket::serde::json::Json;
use rocket::State;

use crate::app_data::TrackerInformation;
use app_data::AppData;

mod app_data;
mod duration_serializer;
mod instant_serializer;
mod tempo_api;

#[get("/")]
fn list(app_data: &State<AppData>) -> Json<Vec<TrackerInformation>> {
    Json(app_data.list_trackers())
}

#[get("/<key>")]
fn elapsed(key: &str, app_data: &State<AppData>) -> Option<String> {
    app_data
        .elapsed_seconds(key)
        .map(|duration| humantime::format_duration(duration).to_string())
}

#[post("/<key>")]
fn create(key: &str, app_data: &State<AppData>) -> Result<(), Conflict<()>> {
    app_data.create_tracker(key).map_err(|_| Conflict(None))
}

#[post("/<key>/start")]
fn start(key: &str, app_data: &State<AppData>) -> Option<()> {
    app_data.start(key)
}

#[post("/pause")]
fn pause(app_data: &State<AppData>) {
    app_data.pause();
}

#[rocket::main]
async fn main() {
    let state = AppData::new();
    let _ = state.create_tracker("a");
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![list, elapsed, create, start, pause])
        .launch()
        .await;
}
