use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerInformation {
    pub key: String,
    pub description: Option<String>,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub running: bool,
    pub start_time: DateTime<Local>,
}
