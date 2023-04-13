extern crate core;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::FromRef;
use axum::Router;

use crate::app_data::AppData;
use crate::config::{get_initial_state, watch_state_file};
use crate::tempo_api::TempoApi;

mod app_data;
mod config;
mod logging;
mod serde;
mod tempo_api;
mod web;

#[derive(Clone)]
pub struct AppState {
    data: Arc<AppData>,
    tempo_api: Arc<TempoApi>,
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
    let state = AppState {
        data: Arc::new(get_initial_state()),
        tempo_api: Arc::new(TempoApi::new()),
    };

    let cloned_state = state.data.clone();

    let _hotwatch = watch_state_file(move || cloned_state.reload_state());

    let app: Router = web::router()
        .layer(logging::logging_layer())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
