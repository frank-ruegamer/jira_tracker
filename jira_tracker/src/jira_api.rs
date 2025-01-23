use crate::config::AppConfig;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, HeaderValue};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;

const BASE_URI: &str = "https://anevis.atlassian.net/rest/api/latest";

#[derive(Debug)]
pub struct JiraApi {
    client: reqwest::Client,
}

impl From<&AppConfig> for JiraApi {
    fn from(value: &AppConfig) -> Self {
        let auth_string = format!("{}:{}", value.jira_email, value.jira_api_token);

        let mut authorization_value: HeaderValue =
            format!("Basic {}", STANDARD.encode(auth_string))
                .parse()
                .unwrap();
        authorization_value.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client }
    }
}

impl JiraApi {
    pub async fn get_account_id(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/myself", BASE_URI);
        let response = self.client.get(&url).send().await?;
        let json = response.json::<serde_json::Value>().await?;
        let account_id = json["accountId"].as_str().unwrap();
        Ok(account_id.to_string())
    }

    pub async fn get_issue_info<K: AsRef<str>>(
        &self,
        issue_key: K,
    ) -> Result<JiraIssue, reqwest::Error> {
        let url = format!("{}/issue/{}", BASE_URI, issue_key.as_ref());
        let response = self
            .client
            .get(&url)
            .query(&[("fields", "summary")])
            .send()
            .await?;
        response.error_for_status()?.json::<JiraIssue>().await
    }
}

#[derive(Debug, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub fields: IssueFields,
}

#[derive(Debug, Deserialize)]
pub struct IssueFields {
    pub summary: String,
}
