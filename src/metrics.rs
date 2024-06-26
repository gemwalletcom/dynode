use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

#[derive(Debug)]
pub struct Metrics {
    registry: Registry,
    proxy_request_counter: Counter<u64>,
    proxy_requests: Family<ProxyStateLabels, Gauge>,
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
        let proxy_request_counter: Counter<u64> = Default::default();
        let proxy_requests = Family::<ProxyStateLabels, Gauge>::default();
        let node_block_latest = Family::<HostStateLabels, Gauge>::default();

        let mut registry = <Registry>::default();
        registry.register(
            "proxy_requests_count",
            "How many requests the application has received",
            proxy_request_counter.clone(),
        );
        registry.register(
            "proxy_requests",
            "Proxy requests by host",
            proxy_requests.clone(),
        );
        registry.register(
            "node_block_latest",
            "Node block latest",
            node_block_latest.clone(),
        );

        Self {
            registry,
            proxy_request_counter,
            proxy_requests,
            node_block_latest,
        }
    }

    pub fn add_total_requests(&self) {
        self.proxy_request_counter.inc();
    }

    pub fn add_proxy_request(&self, host: &str, status: u64, latency: u64) {
        self.proxy_requests
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

    // fn metrics_logger(&self) -> MetricsLogger {
    //     MetricsLogger {
    //         request_counter: &self.request_counter,
    //     }
    // }
}

// pub struct MetricsLogger<'a> {
//     pub request_counter: &'a Counter<u64>,
// }
