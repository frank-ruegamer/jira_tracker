use std::sync::Arc;

use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use serde::Deserialize;

use crate::app_data::{AppData, CreationError, TrackerInformation};
use crate::config::LogError;
use crate::TempoApi;

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
fn create(key: &str, app_data: &AppState) -> Result<(), CreationError> {
    app_data.create_tracker(key)?;
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

pub fn routes() -> Vec<Route> {
    routes![list, get, create, adjust, delete, clear, start, pause, current, submit]
}
