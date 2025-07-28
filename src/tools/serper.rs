use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
  pub title: String,
  pub link: String,
  pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
  #[serde(rename = "searchParameters")]
  pub search_parameters: SearchParameters,
  pub organic: Vec<OrganicResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParameters {
  pub q: String,
  #[serde(rename = "type")]
  pub search_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganicResult {
  pub title: String,
  pub link: String,
  pub snippet: String,
  pub position: i32,
}

pub async fn search_with_key(query: &str, api_key: &str) -> Result<Vec<SearchResult>, reqwest::Error> {
  let client = reqwest::Client::new();

  let response = client
    .post("https://google.serper.dev/search")
    .header("X-API-KEY", api_key)
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
        "q": query,
        "type": "search"
    }))
    .send()
    .await?
    .json::<SearchResponse>()
    .await?;

  let results = response
    .organic
    .into_iter()
    .map(|result| SearchResult {
      title: result.title,
      link: result.link,
      snippet: result.snippet,
    })
    .collect();

  Ok(results)
}

// Backward compatibility function
#[allow(dead_code)]
pub async fn search(query: &str) -> Result<Vec<SearchResult>, reqwest::Error> {
  dotenv::dotenv().ok();
  let api_key = env::var("SERPER_API_KEY").expect("SERPER_API_KEY must be set");
  search_with_key(query, &api_key).await
}

// 将搜索结果格式化为易读的字符串
#[allow(dead_code)]
pub fn format_results(results: &[SearchResult], max_results: usize) -> String {
  let results = results
    .iter()
    .take(max_results)
    .enumerate()
    .map(|(i, result)| {
      format!(
        "{}. {}\n   链接: {}\n   摘要: {}\n",
        i + 1,
        result.title,
        result.link,
        result.snippet
      )
    })
    .collect::<Vec<_>>()
    .join("\n");

  if results.is_empty() {
    "没有找到相关结果。".to_string()
  } else {
    results
  }
}
