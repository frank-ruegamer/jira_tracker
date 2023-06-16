use std::error::Error;
use std::path::PathBuf;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use figment::providers::Env;
use figment::Figment;
use serde::Deserialize;

pub struct LogError(Box<dyn Error>);

impl<E> From<E> for LogError
where
    E: Error + 'static,
{
    fn from(value: E) -> Self {
        LogError(Box::new(value))
    }
}

impl IntoResponse for LogError {
    fn into_response(self) -> Response {
        let LogError(error) = self;
        eprintln!("Internal Server Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub jira_account_id: String,
    pub tempo_api_token: String,
    json_file: String,
}

impl AppConfig {
    pub fn new() -> Self {
        let figment = Figment::from(Env::raw());
        figment.extract().unwrap()
    }

    pub fn get_state_file(&self) -> PathBuf {
        PathBuf::from(shellexpand::full(&self.json_file).unwrap().as_ref())
    }
}
