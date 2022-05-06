#[macro_use]
extern crate rocket;

use ::serde::Deserialize;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use app_data::AppData;
use config::LogError;

use crate::app_data::TrackerInformation;
use crate::tempo_api::TempoApi;

mod app_data;
mod config;
mod serde;
mod tempo_api;

#[get("/trackers")]
fn list(app_data: &State<AppData>) -> Json<Vec<TrackerInformation>> {
    Json(app_data.list_trackers())
}

#[get("/trackers/<key>")]
fn get(key: &str, app_data: &State<AppData>) -> Option<Json<TrackerInformation>> {
    app_data.get_tracker(key).map(|tracker| Json(tracker))
}

#[post("/trackers/<key>")]
fn create(key: &str, app_data: &State<AppData>) -> Result<(), status::Conflict<()>> {
    app_data
        .create_tracker(key)
        .map_err(|_| status::Conflict(None))?;
    app_data.start(key).unwrap();
    Ok(())
}

#[post("/trackers/<key>/start")]
fn start(key: &str, app_data: &State<AppData>) -> Option<()> {
    app_data.start(key)
}

#[derive(Debug, Deserialize)]
struct AdjustDescriptionBody {
    description: Option<String>,
}

#[put("/trackers/<key>", data = "<data>", rank = 2)]
fn adjust(app_data: &State<AppData>, key: &str, data: Json<AdjustDescriptionBody>) -> Option<()> {
    app_data.set_description(key, data.into_inner().description)
}

#[delete("/trackers/<key>")]
fn delete(key: &str, app_data: &State<AppData>) -> Option<status::NoContent> {
    app_data.remove(key).map(|_| status::NoContent)
}

#[delete("/trackers")]
fn clear(app_data: &State<AppData>) -> status::NoContent {
    app_data.remove_all();
    status::NoContent
}

#[get("/tracker")]
fn current(app_data: &State<AppData>) -> Option<Json<TrackerInformation>> {
    app_data.current().map(|tracker| Json(tracker))
}

#[post("/tracker/pause")]
fn pause(app_data: &State<AppData>) {
    app_data.pause();
}

#[post("/submit")]
async fn submit(app_data: &State<AppData>, api: &State<TempoApi>) -> Result<(), LogError> {
    api.submit_all(app_data.remove_all())
        .await
        .map_err(|e| LogError(e))
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .manage(config::get_initial_state())
        .manage(TempoApi::new())
        .mount(
            "/",
            routes![list, get, create, adjust, delete, clear, start, pause, current, submit],
        )
        .launch()
        .await;
}
