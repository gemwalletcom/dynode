mod config;
mod logger;
mod node_service;
mod request_url;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use node_service::NodeService;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = config::NodeConfig::new().unwrap();

    println!("config: {:?}", config);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    let listener = TcpListener::bind(addr).await?;

    let service = NodeService {
        domains: config.domains_map(),
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let service_clone = service.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_clone)
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
