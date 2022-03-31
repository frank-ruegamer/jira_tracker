#[macro_use]
extern crate rocket;

use std::error::Error;

use rocket::http::Status;
use rocket::response::status::Conflict;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{Request, Response, State};

use app_data::AppData;

use crate::app_data::TrackerInformation;
use crate::tempo_api::TempoApi;

mod app_data;
mod duration_serializer;
mod instant_serializer;
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
fn create(key: &str, app_data: &State<AppData>) -> Result<(), Conflict<()>> {
    app_data.create_tracker(key).map_err(|_| Conflict(None))
}

#[post("/trackers/<key>/start")]
fn start(key: &str, app_data: &State<AppData>) -> Option<()> {
    app_data.start(key)
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

struct LogError(Box<dyn Error>);

impl<'r, 'o: 'r> Responder<'r, 'o> for LogError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
        println!("Internal Server Error: {}", self.0);
        Response::build().status(Status::InternalServerError).ok()
    }
}

#[rocket::main]
async fn main() {
    let state = AppData::new();
    let _ = state.create_tracker("WEBAPP-121");
    state.start("WEBAPP-121");
    let _ = rocket::build()
        .manage(state)
        .manage(TempoApi::new())
        .mount("/", routes![list, get, create, start, pause, submit])
        .launch()
        .await;
}
