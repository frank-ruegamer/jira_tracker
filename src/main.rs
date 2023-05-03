extern crate core;

use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;

use axum::extract::FromRef;
use axum::Router;
use hotwatch::Hotwatch;
use tower_http::normalize_path::NormalizePathLayer;

use crate::app_data::AppData;
use crate::config::AppConfig;
use crate::tempo_api::TempoApi;

mod app_data;
mod config;
mod logging;
mod serde;
mod tempo_api;
mod web;

#[derive(Clone)]
pub struct AppState<'a> {
    data: Arc<AppData<'a>>,
    tempo_api: Arc<TempoApi>,
    config: Arc<AppConfig>,
}

impl<'a> From<AppConfig> for AppState<'a> {
    fn from(value: AppConfig) -> Self {
        let config = Arc::new(value);
        AppState {
            data: Arc::new(AppData::from(config.clone().as_ref())),
            tempo_api: Arc::new(TempoApi::from(config.clone().as_ref())),
            config: config,
        }
    }
}

impl<'a> FromRef<AppState<'a>> for Arc<AppData<'a>> {
    fn from_ref(input: &AppState<'a>) -> Self {
        input.data.clone()
    }
}

impl<'a> FromRef<AppState<'a>> for Arc<TempoApi> {
    fn from_ref(input: &AppState) -> Self {
        input.tempo_api.clone()
    }
}

#[tokio::main]
async fn main() {
    let logging_layer = logging::setup_logging();

    let config = AppConfig::new();
    let state = AppState::from(config);
    let cloned_state = state.data.clone();

    let _hotwatch = state
        .config
        .watch_state_file(move || cloned_state.reload_state());

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
