mod chain_service;
mod config;
mod logger;
mod metrics;
mod metrics_service;
mod node_service;
mod proxy_request_service;
mod request_url;

use futures::future::join;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use metrics::Metrics;
use metrics_service::MetricsService;
use node_service::NodeService;
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use tokio::net::TcpListener;
use crate::config::MetricsConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = config::NodeConfig::new()?;

    let node_address = SocketAddr::from((IpAddr::from_str(config.address.as_str())?, config.port));
    let metrics_address = SocketAddr::from((
        IpAddr::from_str(config.metrics.address.as_str())?,
        config.metrics.port,
    ));

    let node_listener = TcpListener::bind(node_address).await?;
    let metrics_listener = TcpListener::bind(metrics_address).await?;

    let metrics_config = MetricsConfig {
        user_agent_patterns: config.metrics.user_agent_patterns.clone(),
    };
    let metrics = Metrics::new(metrics_config);
    let node_service = NodeService::new(config.domains_map(), metrics.clone());
    let node_service_clone = node_service.clone();
    tokio::task::spawn(async move {
        node_service_clone.update_block_numbers().await;
    });

    let node_server = async move {
        loop {
            let (stream, _) = node_listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            let service = node_service.clone().get_proxy_request().await.clone();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    };

    let metrics_server = async move {
        loop {
            let (stream, _) = metrics_listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            let metrics_service = MetricsService {
                metrics: metrics.clone(),
            };

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, metrics_service)
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    };

    println!("Listening node service on http://{}", node_address);
    println!("Listening metrics service on http://{}", metrics_address);

    let _ret = join(node_server, metrics_server).await;

    Ok(())
}
