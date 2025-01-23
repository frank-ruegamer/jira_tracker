use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerInformation {
    pub key: String,
    pub id: String,
    pub description: Option<String>,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub running: bool,
    pub start_time: DateTime<Local>,
}
