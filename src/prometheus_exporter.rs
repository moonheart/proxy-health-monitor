use crate::model::{ProxiesResponse, TestData};
use lazy_static::lazy_static;
use prometheus::{Encoder, GaugeVec, Opts, Registry, TextEncoder};
use prometheus_reqwest_remote_write::WriteRequest;
// Removed: use std::collections::HashMap;
use std::sync::Arc;
// Removed: use warp::Filter;

lazy_static! {
    pub static ref PROXY_DELAY_MS: GaugeVec = GaugeVec::new(
        Opts::new("proxy_delay_ms", "Proxy delay in milliseconds"),
        &["group_name", "proxy_name"]
    )
    .expect("metric can be created: proxy_delay_ms");
}

fn get_latest_delay_from_testdata(test_data: &TestData) -> Option<u32> {
    test_data.history.last().map(|entry| entry.delay)
}

pub fn update_proxy_metrics(
    proxies_response: &ProxiesResponse,
    monitored_groups: &[String],
    config_test_url: &str,
) {
    log::debug!("Updating Prometheus metrics for {} monitored groups.", monitored_groups.len());

    for group_name in monitored_groups {
        if let Some(group_detail) = proxies_response.proxies.get(group_name) {
            if let Some(proxies_in_group) = &group_detail.all {
                log::trace!("Processing group: {}, with {} proxies.", group_name, proxies_in_group.len());
                for proxy_name_in_group in proxies_in_group {
                    if let Some(proxy_detail) = proxies_response.proxies.get(proxy_name_in_group) {
                        let mut latest_delay: Option<u32> = None;

                        // 1. Try to get delay from 'extra' field for the specific test_url
                        if let Some(extra_data) = &proxy_detail.extra {
                            if let Some(test_data_for_url) = extra_data.get(config_test_url) {
                                latest_delay = get_latest_delay_from_testdata(test_data_for_url);
                                log::trace!("Proxy '{}' in group '{}': Found delay {}ms from 'extra' for URL '{}'.", proxy_detail.name, group_name, latest_delay.unwrap_or(0), config_test_url);
                            }
                        }

                        // 2. If not found in 'extra', try to get from top-level 'history'
                        if latest_delay.is_none() {
                            if let Some(history_data) = &proxy_detail.history {
                                latest_delay = history_data.last().map(|entry| entry.delay);
                                log::trace!("Proxy '{}' in group '{}': Found delay {}ms from top-level 'history'.", proxy_detail.name, group_name, latest_delay.unwrap_or(0));
                            }
                        }
                        
                        // If delay is 0 and proxy is not alive, it might be a real 0ms (e.g. DIRECT) or a failed test.
                        // Mihomo reports 0 for failed tests. We'll report 0 as is.
                        // If a proxy is 'alive: false' but has a non-zero delay, it's contradictory.
                        // We prioritize the delay value from history if available.
                        // If no history, and not alive, perhaps a very high value or skip? For now, report 0 if no history.
                        let delay_to_report = latest_delay.unwrap_or(0);

                        PROXY_DELAY_MS
                            .with_label_values(&[group_name, &proxy_detail.name])
                            .set(delay_to_report as f64);
                        log::debug!(
                            "Set metric: group_name='{}', proxy_name='{}', delay_ms={}",
                            group_name,
                            proxy_detail.name,
                            delay_to_report
                        );
                    } else {
                        log::warn!(
                            "Proxy '{}' listed in group '{}' not found in main proxies list.",
                            proxy_name_in_group,
                            group_name
                        );
                    }
                }
            } else {
                log::warn!("Monitored group '{}' does not have an 'all' field or is not a group type proxy. Skipping.", group_name);
            }
        } else {
            log::warn!("Monitored group '{}' not found in proxies response.", group_name);
        }
    }
}


pub async fn metrics_handler(registry: Arc<Registry>) -> Result<impl warp::Reply, warp::Rejection> {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    if let Err(e) = encoder.encode(&registry.gather(), &mut buffer) {
        log::error!("Could not encode prometheus metrics: {}", e);
        // Return internal server error
        let mut resp = warp::reply::Response::new("Internal Server Error".into());
        *resp.status_mut() = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(resp);
    }
    let mut resp = warp::reply::Response::new(buffer.into());
    resp.headers_mut().insert(
        "Content-Type",
        warp::http::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    Ok(resp)
}

// Basic push functionality (Prometheus crate's push_to_gateway is often for Pushgateway, not direct remote write)
// For direct remote write, one might need to format according to Prometheus remote write spec
// or use a crate that specifically supports it. This is a simplified version.
pub async fn push_metrics_to_remote_write(
    push_url: &str,
    registry: &Registry,
    client: &reqwest::Client,
) -> Result<(), String> {
    let write_request = WriteRequest::from_metric_families(registry.gather(), None).expect("Failed to create WriteRequest from metrics");
    let http_request = write_request.build_http_request(client.clone(), push_url, "proxy_health_monitor").expect("Failed to build HTTP request for remote write");
    match client.execute(http_request).await {
        Ok(response) => {
            if response.status().is_success() {
                log::info!("Successfully pushed metrics to remote write endpoint: {}", push_url);
                Ok(())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "No response body".to_string());
                log::error!("Failed to push metrics to remote write endpoint: {}. Status: {}, Body: {}", push_url, status, body);
                Err(format!("Failed to push metrics: {} - {}", status, body))
            }
        }
        Err(e) => {
            log::error!("Error pushing metrics to remote write endpoint '{}': {}", push_url, e);
            Err(format!("Error pushing metrics: {}", e))
        }
    }
        
}

pub fn register_metrics(registry: &Registry) {
    if let Err(e) = registry.register(Box::new(PROXY_DELAY_MS.clone())) {
        log::error!("Failed to register PROXY_DELAY_MS metric: {:?}", e);
    }
}