use std::env;
use std::error::Error;

use futures::future::try_join_all;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Serialize;

use crate::app_data::PausedTracker;

const JIRA_ACCOUNT_ID: &str = "JIRA_ACCOUNT_ID";
const TEMPO_API_TOKEN: &str = "TEMPO_API_TOKEN";

pub struct TempoApi {
    client: reqwest::Client,
    jira_account_id: String,
}

#[derive(Debug, Serialize)]
struct SubmitWorklogBody {
    #[serde(rename = "issueKey")]
    issue_key: String,
    #[serde(rename = "timeSpentSeconds")]
    time_spent_seconds: u64,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "authorAccountId")]
    author_account_id: String,
}

impl SubmitWorklogBody {
    fn from(tracker: PausedTracker, author_account_id: &str) -> Self {
        SubmitWorklogBody {
            issue_key: tracker.key,
            time_spent_seconds: tracker.duration.as_secs(),
            start_date: tracker.start_time.format("%Y-%m-%d").to_string(),
            start_time: tracker.start_time.format("%H:%M:%S").to_string(),
            author_account_id: author_account_id.to_string(),
        }
    }
}

impl TempoApi {
    pub fn new() -> Self {
        let jira_account_id = env::var(JIRA_ACCOUNT_ID).unwrap();
        let tempo_api_token = env::var(TEMPO_API_TOKEN).unwrap();
        let mut authorization_value: HeaderValue =
            format!("Bearer {}", tempo_api_token).parse().unwrap();
        authorization_value.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            client,
            jira_account_id,
        }
    }

    pub async fn submit(&self, tracker: PausedTracker) -> Result<(), Box<dyn Error>> {
        let request: SubmitWorklogBody = SubmitWorklogBody::from(tracker, &self.jira_account_id);
        let builder = self
            .client
            .post("https://api.tempo.io/core/3/worklogs")
            .json(&request);
        builder.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn submit_all(&self, trackers: Vec<PausedTracker>) -> Result<(), Box<dyn Error>> {
        let results: Vec<_> = trackers
            .into_iter()
            .map(|tracker| self.submit(tracker))
            .collect();
        try_join_all(results).await.map(|_| ())
    }
}
