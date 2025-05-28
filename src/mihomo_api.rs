use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use crate::model::ProxiesResponse;

/// 触发指定代理组的延迟测试
pub async fn trigger_delay_test(
    client: &reqwest::Client,
    api_url: &str,
    api_secret: &str,
    group_name: &str,
    test_url: &str,
    timeout_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/group/{}/delay?url={}&timeout={}",
        api_url.trim_end_matches('/'),
        group_name,
        test_url,
        timeout_ms
    );

    let mut headers = HeaderMap::new();
    if !api_secret.is_empty() {
        let bearer_token = format!("Bearer {}", api_secret);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer_token)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?,
        );
    }

    log::debug!("Triggering delay test for group '{}' with URL: {}", group_name, url);
    let response = client.get(&url).headers(headers).send().await?;

    if response.status().is_success() {
        log::info!("Successfully triggered delay test for group: {}", group_name);
        Ok(())
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
        log::error!(
            "Failed to trigger delay test for group {}: {} - {}",
            group_name,
            status,
            text
        );
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("API request failed with status {}: {}", status, text),
        )))
    }
}

/// 获取所有代理的详细信息
pub async fn get_proxies_info(
    client: &reqwest::Client,
    api_url: &str,
    api_secret: &str,
) -> Result<ProxiesResponse, Box<dyn std::error::Error>> {
    let url = format!("{}/proxies", api_url.trim_end_matches('/'));
    
    log::info!("Fetching proxies info from URL: {}", url);

    let mut headers = HeaderMap::new();
    if !api_secret.is_empty() {
        let bearer_token = format!("Bearer {}", api_secret);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer_token)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?,
        );
    }
    
    log::debug!("Fetching proxies info from URL: {}", url);
    let response = client.get(&url).headers(headers).send().await?;

    if response.status().is_success() {
        let proxies_response = response.json::<ProxiesResponse>().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        log::info!("Successfully fetched proxies info.");
        Ok(proxies_response)
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
        log::error!("Failed to fetch proxies info: {} - {}", status, text);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("API request failed with status {}: {}", status, text),
        )))
    }
}