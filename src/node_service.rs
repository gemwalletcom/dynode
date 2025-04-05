use std::str::FromStr;
use std::{collections::HashMap, sync::Arc, time::Instant};

use futures::future;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::config::Url;
use crate::metrics::Metrics;
use primitives::ChainType;
use crate::{
    chain_service::ChainService,
    config::Domain,
    proxy_request_service::{NodeDomain, ProxyRequestService},
};

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
    pub nodes: Arc<Mutex<HashMap<String, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
}

#[derive(Debug)]
pub struct NodeRawResult {
    pub url: Url,
    pub result: Result<u64, Box<dyn std::error::Error + Send + Sync>>,
    pub latency: u64,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub url: Url,
    pub block_number: u64,
    pub latency: u64,
}

impl NodeService {
    pub fn new(domains: HashMap<String, Domain>, metrics: Metrics) -> Self {
        //
        let mut hash_map: HashMap<String, NodeDomain> = HashMap::new();

        for (key, domain) in domains.clone() {
            let url = domain.urls.first().unwrap().clone();
            hash_map.insert(key, NodeDomain { url });
        }

        Self {
            domains,
            nodes: Arc::new(Mutex::new(hash_map)),
            metrics: Arc::new(metrics),
        }
    }

    pub async fn get_proxy_request(&self) -> ProxyRequestService {
        ProxyRequestService {
            domains: self.get_node_domains().await,
            metrics: self.metrics.as_ref().clone(),
        }
    }

    pub async fn get_node_domain(
        nodes: &Arc<Mutex<HashMap<String, NodeDomain>>>,
        domain: String,
    ) -> Option<NodeDomain> {
        (nodes.lock().await).get(&domain).cloned()
    }

    pub async fn update_node_domain(
        nodes: &Arc<Mutex<HashMap<String, NodeDomain>>>,
        domain: String,
        node_domain: NodeDomain,
    ) {
        let mut map = nodes.lock().await;
        map.insert(domain, node_domain);
    }

    pub async fn get_node_domains(&self) -> HashMap<String, NodeDomain> {
        (*self.nodes.lock().await).clone()
    }

    pub async fn update_block_numbers(&self) {
        for (_, domain) in self.domains.clone() {
            self.metrics
                .set_node_host_current(&domain.domain, &domain.urls.first().unwrap().url);

            if domain.urls.len() > 1 {
                let domain = domain.clone();

                let nodes = Arc::clone(&self.nodes);
                //let metrics = Arc::clone(&self.metrics);

                tokio::task::spawn(async move {
                    loop {
                        let tasks: Vec<_> = domain
                            .clone()
                            .urls
                            .iter()
                            .flat_map(|url| {
                                let chain_type = domain.chain_type.clone();
                                let url = url.clone();
                                if let Some(chain_type) = ChainType::from_str(&chain_type).ok() {
                                    Some(tokio::spawn(async move {
                                        let now = Instant::now();
                                        let result = Self::get_latest_block(chain_type, &url.url.as_str()).await;

                                        NodeRawResult {
                                            url: url.clone(),
                                            result,
                                            latency: now.elapsed().as_millis() as u64,
                                        }
                                    }))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let results: Vec<NodeResult> = future::join_all(tasks)
                            .await
                            .into_iter()
                            .filter_map(|res| res.ok())
                            .filter_map(|res| {
                                res.result.ok().map(|block_number| NodeResult {
                                    url: res.url,
                                    block_number,
                                    latency: res.latency,
                                })
                            })
                            .collect();

                        if let Some(value) =
                            Self::get_node_domain(&nodes.clone(), domain.domain.clone()).await
                        {
                            let is_url_behind =
                                domain.is_url_behind(value.url.clone(), results.clone());

                            println!(
                                "node service: {} is behind: {}, {:?}",
                                value.url.url,
                                is_url_behind,
                                results
                                    .clone()
                                    .into_iter()
                                    .map(|x| x.block_number)
                                    .collect::<Vec<u64>>()
                            );
                            if is_url_behind {
                                if let Some(node) =
                                    Domain::find_highest_block_number(results.clone())
                                {
                                    Self::update_node_domain(
                                        &nodes,
                                        domain.domain.clone(),
                                        NodeDomain {
                                            url: node.url.clone(),
                                        },
                                    )
                                    .await;

                                    println!(
                                        "node service: {:?} set new node: {}, latest: {}mc",
                                        domain.domain.clone(),
                                        node.url.url.clone(),
                                        node.latency
                                    );
                                }
                            }
                        }

                        sleep(Duration::from_secs(domain.get_poll_interval_seconds())).await;
                    }
                });
            }
        }
    }

    pub async fn get_latest_block(
        chain_type: ChainType,
        url: &str,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let chain_service = ChainService {
            chain_type: chain_type.clone(),
            url: url.to_string(),
        };
        chain_service.get_block_number().await
    }

    #[allow(dead_code)]
    pub async fn update_latest_block(chain_type: ChainType, url: &str) {
        let chain_service = ChainService {
            chain_type: chain_type.clone(),
            url: url.to_string(),
        };
        let now = Instant::now();
        let res = chain_service.get_block_number().await;

        println!(
            "update_latest_block: chain_type: {:?}, url: {} {:?}, {}ms",
            chain_type,
            url,
            res,
            now.elapsed().as_millis()
        );
    }
}
