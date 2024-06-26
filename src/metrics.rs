use std::sync::Arc;

use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

#[derive(Debug, Clone)]
pub struct Metrics {
    registry: Arc<Registry>,
    proxy_requests: Family<HostStateLabels, Gauge>,
    proxy_responses: Family<ProxyStateLabels, Gauge>,
    node_block_latest: Family<HostStateLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(crate) struct HostStateLabels {
    host: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(crate) struct ProxyStateLabels {
    host: String,
    status: u64,
    latency: u64,
}

impl Metrics {
    pub fn new() -> Self {
        let proxy_requests = Family::<HostStateLabels, Gauge>::default();
        let proxy_responses = Family::<ProxyStateLabels, Gauge>::default();
        let node_block_latest = Family::<HostStateLabels, Gauge>::default();

        let mut registry = <Registry>::with_prefix("dynode");
        registry.register(
            "proxy_requests_total",
            "Proxy requests by host",
            proxy_requests.clone(),
        );
        registry.register(
            "proxy_responses_total",
            "Proxy requests served a response by host",
            proxy_responses.clone(),
        );
        registry.register(
            "node_block_latest",
            "Node block latest",
            node_block_latest.clone(),
        );

        Self {
            registry: Arc::new(registry),
            proxy_requests,
            proxy_responses,
            node_block_latest,
        }
    }

    pub fn add_proxy_request(&self, host: &str) {
        self.proxy_requests
            .get_or_create(&HostStateLabels {
                host: host.to_string(),
            })
            .inc();
    }

    pub fn add_proxy_response(&self, host: &str, status: u64, latency: u64) {
        self.proxy_responses
            .get_or_create(&ProxyStateLabels {
                host: host.to_string(),
                status,
                latency,
            })
            .inc();
    }

    pub fn set_node_block_latest(&self, host: &str, value: u64) {
        self.node_block_latest
            .get_or_create(&HostStateLabels {
                host: host.to_string(),
            })
            .set(value as i64);
    }

    pub fn get_metrics(&self) -> String {
        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }
}
