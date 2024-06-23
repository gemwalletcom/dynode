use prometheus_client::metrics::counter::Counter;
use prometheus_client::registry::Registry;

#[derive(Debug)]
pub struct Metrics {
    registry: Registry,
    pub request_counter: Counter<u64>,
}

impl Metrics {
    pub fn new() -> Self {
        let request_counter: Counter<u64> = Default::default();

        let mut registry = <Registry>::default();
        registry.register(
            "requests",
            "How many requests the application has received",
            request_counter.clone(),
        );

        Self {
            registry,
            request_counter,
        }
    }

    fn add_total_requests(&self) {
        self.request_counter.inc();
    }

    fn metrics_logger(&self) -> MetricsLogger {
        MetricsLogger {
            request_counter: &self.request_counter,
        }
    }
}

// pub struct MetricsLogger<'a> {
//     pub request_counter: &'a Counter<u64>,
// }
