use std::time::Duration;

use futures::future::try_join_all;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Serialize;

use crate::config::AppConfig;
use domain::TrackerInformation;

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
    description: Option<String>,
    #[serde(rename = "authorAccountId")]
    author_account_id: String,
}

impl<ID> From<(TrackerInformation, ID)> for SubmitWorklogBody
where
    ID: Into<String>,
{
    fn from((tracker, author_account_id): (TrackerInformation, ID)) -> Self {
        Self {
            issue_key: tracker.key,
            time_spent_seconds: tracker.duration.as_secs(),
            start_date: tracker.start_time.format("%Y-%m-%d").to_string(),
            start_time: tracker.start_time.format("%H:%M:%S").to_string(),
            description: tracker.description,
            author_account_id: author_account_id.into(),
        }
    }
}

impl TempoApi {
    fn new(tempo_api_token: &str, jira_account_id: &str) -> Self {
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
            jira_account_id: jira_account_id.to_string(),
        }
    }

    pub async fn submit(&self, tracker: TrackerInformation) -> Result<(), reqwest::Error> {
        let request: SubmitWorklogBody = (tracker, &self.jira_account_id).into();
        let builder = self
            .client
            .post("https://api.tempo.io/core/3/worklogs")
            .json(&request);
        builder.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn submit_all(
        &self,
        trackers: Vec<TrackerInformation>,
    ) -> Result<(), reqwest::Error> {
        let results: Vec<_> = trackers
            .into_iter()
            .filter(|tracker| tracker.duration >= Duration::from_secs(60))
            .map(|tracker| self.submit(tracker))
            .collect();
        try_join_all(results).await.map(|_| ())
    }
}

impl From<&AppConfig> for TempoApi {
    fn from(config: &AppConfig) -> Self {
        TempoApi::new(&config.tempo_api_token, &config.jira_account_id)
    }
}
