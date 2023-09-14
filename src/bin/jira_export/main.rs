use std::fs;

use futures::future::try_join_all;
use serde_json::{to_string, Value};

const URL: &str = "https://anevis.atlassian.net/rest/api/latest/search";
const PAGE_SIZE: usize = 100;

fn read_file(path: &str) -> String {
    fs::read_to_string(dirs::home_dir().unwrap().join(path))
        .unwrap()
        .trim()
        .to_string()
}

#[tokio::main]
async fn main() {
    let user = &read_file(".jira/email");
    let token = &read_file(".jira/token");
    let client = reqwest::Client::builder().build().unwrap();

    let request = |params: Vec<(&str, &str)>| {
        let url = reqwest::Url::parse_with_params(URL, params).unwrap();
        async {
            let builder = client.get(url).basic_auth(user, Some(token));
            let response = builder.send().await.unwrap();
            response.json::<Value>().await
        }
    };

    let res = request(vec![("maxResults", "0")]).await;
    let response = res.unwrap();
    let total = response["total"].as_u64().unwrap();

    let req = |start: u64| async move {
        let start_at: &str = &start.to_string();
        let page_size: &str = &PAGE_SIZE.to_string();
        let params = vec![
            ("startAt", start_at),
            ("maxResults", page_size),
            ("fields", "summary,status"),
            ("jql", "order by key"),
        ];
        let value = request(params).await;
        value.map(|v| v["issues"].as_array().unwrap().to_vec())
    };

    let futures: Vec<_> = (0..total)
        .step_by(PAGE_SIZE)
        .map(|start| req(start))
        .collect();
    let result = try_join_all(futures).await.unwrap();
    let vv: Vec<_> = result.into_iter().flatten().collect();
    println!("{}", to_string(&vv).unwrap());
}
