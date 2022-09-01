#[macro_use]
extern crate rocket;

use std::sync::Arc;

use crate::config::{get_initial_state, watch_state_file};
use crate::tempo_api::TempoApi;

mod app_data;
mod config;
mod serde;
mod tempo_api;
mod web;

#[rocket::main]
async fn main() {
    let state = Arc::new(get_initial_state());
    let cloned_state = state.clone();

    let _hotwatch = watch_state_file(move || cloned_state.reload_state());

    let _ = rocket::build()
        .manage(state)
        .manage(TempoApi::new())
        .mount("/", web::routes())
        .launch()
        .await;
}
