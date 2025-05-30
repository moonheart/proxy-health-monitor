use crate::config::Config;
use crate::mihomo_api;
use crate::prometheus_exporter;
use prometheus::Registry;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub async fn run_scheduler(config: Arc<Config>, registry: Arc<Registry>, http_client: Arc<reqwest::Client>) {
    let mut ticker = interval(Duration::from_secs(config.interval_seconds));
    log::info!(
        "Scheduler started. Will run health checks every {} seconds.",
        config.interval_seconds
    );

    loop {
        ticker.tick().await;
        log::info!("Starting proxy health check cycle...");

        for group_name in &config.groups_to_monitor {
            log::info!("Triggering delay test for group: {}", group_name);
            match mihomo_api::trigger_delay_test(
                &http_client,
                &config.api_url,
                &config.api_secret,
                group_name,
                &config.test_url,
                config.test_timeout_seconds * 1000, // API expects milliseconds
            )
            .await
            {
                Ok(_) => log::info!("Successfully triggered delay test for group: {}", group_name),
                Err(e) => log::error!(
                    "Failed to trigger delay test for group {}: {}",
                    group_name,
                    e
                ),
            }
        }

        // Wait for a moment to allow mihomo to complete tests before fetching results.
        // A more robust solution might involve checking mihomo's status or having a configurable delay.
        let wait_duration = if config.test_timeout_seconds > 1 {
            config.test_timeout_seconds / 2 
        } else {
            1 // Minimum 1 second wait
        };
        log::debug!("Waiting for {} seconds for tests to complete...", wait_duration);
        tokio::time::sleep(Duration::from_secs(wait_duration)).await;

        log::info!("Fetching proxies info...");
        match mihomo_api::get_proxies_info(&http_client, &config.api_url, &config.api_secret).await {
            Ok(proxies_info) => {
                log::info!("Successfully fetched proxies info. Updating Prometheus metrics...");
                prometheus_exporter::update_proxy_metrics(
                    &proxies_info,
                    &config.groups_to_monitor,                    
                    &config.test_url,
                    &config.reporter.as_deref().unwrap_or("unknown"),
                );

                if config.prometheus.mode == "push" {
                    if let Some(push_url) = &config.prometheus.push_url {
                        log::info!("Pushing metrics to Prometheus remote write: {}", push_url);
                        if let Err(e) = prometheus_exporter::push_metrics_to_remote_write(
                            push_url,
                            &registry,
                            &http_client,
                        )
                        .await
                        {
                            log::error!("Failed to push metrics: {}", e);
                        }
                    } else {
                        log::warn!("Prometheus mode is 'push', but no 'push_url' is configured.");
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to fetch proxies info: {}", e);
            }
        }
        log::info!("Proxy health check cycle finished.");
    }
}