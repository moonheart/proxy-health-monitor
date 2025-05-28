mod config;
mod mihomo_api;
mod model;
mod prometheus_exporter;
mod scheduler;

use prometheus::Registry;
use std::sync::Arc;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Initialize logger

    log::info!("Starting Mihomo Proxy Health Monitor...");

    // Load configuration
    let app_config = match config::Config::load_config("config.toml") {
        Ok(cfg) => Arc::new(cfg),
        Err(e) => {
            log::error!("Failed to load configuration from config.toml: {}", e);
            // Attempt to create a default config.toml if it doesn't exist or is invalid
            // For simplicity, we'll just exit here. A more robust app might create a default.
            eprintln!("Error: Could not load config.toml. Please ensure it exists and is valid.");
            eprintln!("Details: {}", e);
            std::process::exit(1);
        }
    };
    log::info!("Configuration loaded successfully.");

    // Create Prometheus registry and register metrics
    let registry = Arc::new(Registry::new());
    prometheus_exporter::register_metrics(&registry);
    log::info!("Prometheus metrics registered.");

    // Create a shared reqwest client
    let http_client = Arc::new(reqwest::Client::new());

    // Start metrics server if in pull mode
    if app_config.prometheus.mode == "pull" {
        let listen_address_str = app_config.prometheus.listen_address.clone();
        let registry_clone = Arc::clone(&registry);
        
        tokio::spawn(async move {
            log::info!("Prometheus pull mode enabled. Starting metrics server on http://{}", listen_address_str);
            let metrics_route = warp::path("metrics")
                .and(warp::get()) // Ensure it's a GET request
                .map(move || Arc::clone(&registry_clone))
                .and_then(prometheus_exporter::metrics_handler);

            match listen_address_str.parse::<std::net::SocketAddr>() {
                Ok(socket_addr) => {
                    warp::serve(metrics_route).run(socket_addr).await;
                }
                Err(e) => {
                    log::error!("Invalid listen_address for metrics server '{}': {}. Metrics server will not start.", listen_address_str, e);
                }
            }
        });
    } else if app_config.prometheus.mode == "push" {
        log::info!("Prometheus push mode enabled.");
        if app_config.prometheus.push_url.is_none() {
            log::warn!("Prometheus mode is 'push', but 'push_url' is not configured in config.toml. Metrics will not be pushed.");
        }
    } else {
        log::warn!("Unknown Prometheus mode: '{}'. Metrics will not be exported.", app_config.prometheus.mode);
    }

    // Start the scheduler
    log::info!("Starting health check scheduler...");
    scheduler::run_scheduler(Arc::clone(&app_config), Arc::clone(&registry), http_client).await;
    
    // The scheduler runs indefinitely, so this part might not be reached unless scheduler exits.
    log::info!("Mihomo Proxy Health Monitor shutting down.");
    Ok(())
}
