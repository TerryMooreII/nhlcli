use serde_json::Value;

pub const NHL_API_URL: &'static str = "https://api-web.nhle.com/v1";

pub async fn nhl_api_request(
    client: &reqwest::Client,
    url: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    let json: Value = response.json().await?;
    Ok(json)
} 