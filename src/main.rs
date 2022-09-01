#[macro_use]
extern crate rocket;

use std::sync::Arc;

use ::serde::Deserialize;
use hotwatch::notify::DebouncedEvent;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use app_data::AppData;
use config::{watch_config_file, LogError};

use crate::app_data::TrackerInformation;
use crate::config::get_initial_state;
use crate::tempo_api::TempoApi;

mod app_data;
mod config;
mod serde;
mod tempo_api;

type AppState = State<Arc<AppData>>;

#[get("/trackers")]
fn list(app_data: &AppState) -> Json<Vec<TrackerInformation>> {
    Json(app_data.list_trackers())
}

#[get("/trackers/<key>")]
fn get(key: &str, app_data: &AppState) -> Option<Json<TrackerInformation>> {
    app_data.get_tracker(key).map(|tracker| Json(tracker))
}

#[post("/trackers/<key>")]
fn create(key: &str, app_data: &AppState) -> Result<(), status::Conflict<()>> {
    app_data
        .create_tracker(key)
        .map_err(|_| status::Conflict(None))?;
    app_data.start(key).unwrap();
    Ok(())
}

#[post("/trackers/<key>/start")]
fn start(key: &str, app_data: &AppState) -> Option<()> {
    app_data.start(key)
}

#[derive(Debug, Deserialize)]
struct AdjustDescriptionBody {
    description: Option<String>,
}

#[put("/trackers/<key>", data = "<data>", rank = 2)]
fn adjust(app_data: &AppState, key: &str, data: Json<AdjustDescriptionBody>) -> Option<()> {
    app_data.set_description(key, data.into_inner().description)
}

#[delete("/trackers/<key>")]
fn delete(key: &str, app_data: &AppState) -> Option<status::NoContent> {
    app_data.remove(key).map(|_| status::NoContent)
}

#[delete("/trackers")]
fn clear(app_data: &AppState) -> status::NoContent {
    app_data.remove_all();
    status::NoContent
}

#[get("/tracker")]
fn current(app_data: &AppState) -> Option<Json<TrackerInformation>> {
    app_data.current().map(|tracker| Json(tracker))
}

#[post("/tracker/pause")]
fn pause(app_data: &AppState) {
    app_data.pause()
}

#[post("/submit")]
async fn submit(app_data: &AppState, api: &State<TempoApi>) -> Result<(), LogError> {
    api.submit_all(app_data.list_trackers())
        .await
        .map(|_| {
            app_data.remove_all();
        })
        .map_err(|e| LogError(e))
}

#[rocket::main]
async fn main() {
    let state = Arc::new(get_initial_state());
    let cloned_state = state.clone();

    let _hotwatch = watch_config_file(move || cloned_state.refresh_config());

    let _ = rocket::build()
        .manage(state)
        .manage(TempoApi::new())
        .mount(
            "/",
            routes![list, get, create, adjust, delete, clear, start, pause, current, submit],
        )
        .launch()
        .await;
}
