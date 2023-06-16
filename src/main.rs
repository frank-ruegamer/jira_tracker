#![allow(clippy::new_without_default)]
extern crate core;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::FromRef;
use axum::Router;
use tower_http::normalize_path::NormalizePathLayer;

use crate::app_data::AppData;
use crate::config::AppConfig;
use crate::tempo_api::TempoApi;

mod app_data;
mod config;
mod files;
mod serde;
mod tempo_api;
mod web;

#[derive(Clone)]
pub struct AppState {
    data: Arc<AppData>,
    tempo_api: Arc<TempoApi>,
}

impl From<&AppConfig> for AppState {
    fn from(config: &AppConfig) -> Self {
        AppState {
            data: Arc::new(config.into()),
            tempo_api: Arc::new(config.into()),
        }
    }
}

impl FromRef<AppState> for Arc<AppData> {
    fn from_ref(input: &AppState) -> Self {
        input.data.clone()
    }
}

impl FromRef<AppState> for Arc<TempoApi> {
    fn from_ref(input: &AppState) -> Self {
        input.tempo_api.clone()
    }
}

#[tokio::main]
async fn main() {
    let logging_layer = config::setup_logging();

    let config = &AppConfig::new();
    let state: AppState = config.into();
    let cloned_state = state.data.clone();

    let _hotwatch = files::watch_file(config.get_state_file(), move || cloned_state.reload_state());

    let app: Router = web::router()
        .layer(logging_layer)
        .layer(NormalizePathLayer::trim_trailing_slash())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
