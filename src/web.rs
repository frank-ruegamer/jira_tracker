use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::app_data::{AppData, TrackerError, TrackerInformation};
use crate::config::LogError;
use crate::tempo_api::TempoApi;
use crate::AppState;

async fn list(State(state): State<Arc<AppData>>) -> Json<Vec<TrackerInformation>> {
    Json(state.list_trackers())
}

async fn get_tracker(
    Path(key): Path<String>,
    State(state): State<Arc<AppData>>,
) -> Result<Json<TrackerInformation>, TrackerError> {
    state.get_tracker(&key).map(Json)
}

async fn create(
    Path(key): Path<String>,
    State(state): State<Arc<AppData>>,
) -> Result<(), TrackerError> {
    state.create_tracker(&key)?;
    state.start(&key).unwrap();
    Ok(())
}

async fn start(
    Path(key): Path<String>,
    State(state): State<Arc<AppData>>,
) -> Result<(), TrackerError> {
    state.start(&key)
}

#[derive(Debug, Deserialize)]
struct AdjustDescriptionBody {
    description: Option<String>,
}

async fn adjust(
    Path(key): Path<String>,
    State(state): State<Arc<AppData>>,
    Json(data): Json<AdjustDescriptionBody>,
) -> Result<(), TrackerError> {
    state.set_description(&key, data.description)
}

async fn delete(
    Path(key): Path<String>,
    State(state): State<Arc<AppData>>,
) -> Result<StatusCode, TrackerError> {
    state.remove(&key).map(|_| StatusCode::NO_CONTENT)
}

async fn clear(State(state): State<Arc<AppData>>) -> StatusCode {
    state.remove_all();
    StatusCode::NO_CONTENT
}

async fn current(
    State(state): State<Arc<AppData>>,
) -> Result<Json<TrackerInformation>, TrackerError> {
    state.current().map(Json)
}

async fn pause(State(state): State<Arc<AppData>>) {
    state.pause()
}

#[derive(Debug, Serialize)]
struct SumResponse {
    #[serde(with = "humantime_serde")]
    duration: Duration,
}

async fn sum(State(state): State<Arc<AppData>>) -> Json<SumResponse> {
    Json(SumResponse {
        duration: state.sum(),
    })
}

async fn submit(
    State(state): State<Arc<AppData>>,
    State(api): State<Arc<TempoApi>>,
) -> Result<(), LogError> {
    api.submit_all(state.list_trackers())
        .await
        .map(|_| {
            state.remove_all();
        })
        .map_err(|e| e.into())
}

pub fn router() -> Router<AppState> {
    let trackers_routes = Router::new()
        .route("/", get(list).delete(clear))
        .route(
            "/:key",
            get(get_tracker).post(create).put(adjust).delete(delete),
        )
        .route("/:key/start", post(start));

    let tracker_routes = Router::new()
        .route("/", get(current))
        .route("/pause", post(pause));

    Router::new()
        .nest("/trackers", trackers_routes)
        .nest("/tracker", tracker_routes)
        .route("/sum", get(sum))
        .route("/submit", post(submit))
}
