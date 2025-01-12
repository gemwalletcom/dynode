use std::sync::Arc;

use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;

use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;

#[derive(Debug, Clone)]
pub struct Metrics {
    registry: Arc<Registry>,
    proxy_requests: Family<HostStateLabels, Counter>,
    proxy_response_latency: Family<ResponseLabels, Histogram>,
    node_host_current: Family<HostCurrentStateLabels, Gauge>,
    node_block_latest: Family<HostStateLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostStateLabels {
    host: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostCurrentStateLabels {
    host: String,
    remote_host: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct ResponseLabels {
    host: String,
    remote_host: String,
    path: String,
    status: u16,
    user_agent: String,
}

impl Metrics {
    pub fn new() -> Self {
        let proxy_requests = Family::<HostStateLabels, Counter>::default();
        let proxy_response_latency =
            Family::<ResponseLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(50.0, 1.44, 12))
            });
        let node_host_current = Family::<HostCurrentStateLabels, Gauge>::default();
        let node_block_latest = Family::<HostStateLabels, Gauge>::default();

        let mut registry = <Registry>::with_prefix("dynode");
        registry.register(
            "proxy_requests",
            "Proxy requests by host",
            proxy_requests.clone(),
        );
        registry.register(
            "proxy_response_latency",
            "Proxy requests served a response by host",
            proxy_response_latency.clone(),
        );
        registry.register(
            "node_host_current",
            "Node current host url",
            node_host_current.clone(),
        );
        registry.register(
            "node_block_latest",
            "Node block latest",
            node_block_latest.clone(),
        );

        Self {
            registry: Arc::new(registry),
            proxy_requests,
            proxy_response_latency,
            node_host_current,
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

    fn truncate_path(&self, path: &str) -> String {
        path.split('/')
            .map(|segment| {
                if segment.len() > 20 {
                    ":value".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn add_proxy_response(
        &self,
        host: &str,
        path: &str,
        remote_host: &str,
        user_agent: &str,
        status: u16,
        latency: u128,
    ) {
        let path = self.truncate_path(path);
        self.proxy_response_latency
            .get_or_create(&ResponseLabels {
                host: host.to_string(),
                path,
                remote_host: remote_host.to_string(),
                status,
                user_agent: user_agent.to_string(),
            })
            .observe(latency as f64);
    }

    pub fn set_node_host_current(&self, host: &str, remote_host: &str) {
        self.node_host_current
            .get_or_create(&HostCurrentStateLabels {
                host: host.to_string(),
                remote_host: remote_host.to_string(),
            })
            .set(1);
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
