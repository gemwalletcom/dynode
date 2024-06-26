#![allow(unused)]

use std::{collections::HashMap, env, hash::Hasher};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use crate::{chain_type::ChainType, node_service::NodeResult, proxy_request_service::NodeDomain};

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: Metrics,
    pub domains: Vec<Domain>,
}

impl NodeConfig {
    pub fn domains_map(&self) -> HashMap<String, Domain> {
        let mut map: HashMap<String, Domain> = HashMap::new();
        for domain in &self.domains {
            map.insert(domain.domain.clone(), domain.clone());
        }
        map
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Metrics {
    pub port: u16,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Domain {
    pub domain: String,
    pub chain_type: ChainType,
    pub block_delay: Option<u64>,
    pub poll_interval_seconds: Option<u64>,
    pub urls: Vec<Url>,
}

impl Domain {
    pub fn get_poll_interval_seconds(&self) -> u64 {
        self.poll_interval_seconds.unwrap_or(600) // 10 minutes
    }

    pub fn get_block_delay(&self) -> u64 {
        self.block_delay.unwrap_or(100)
    }

    pub fn get_node_domain(&self, url: Url) -> NodeDomain {
        NodeDomain { url }
    }

    pub fn is_url_behind(&self, url: Url, results: Vec<NodeResult>) -> bool {
        if let Some(index) = results.iter().position(|r| r.url == url) {
            let node = results[index].clone();
            if let Some(max_block_number) = Self::find_highest_block_number(results) {
                if node.block_number + self.get_block_delay() >= max_block_number.block_number {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_highest_block_number(results: Vec<NodeResult>) -> Option<NodeResult> {
        results
            .into_iter()
            .max_by(|x, y| x.block_number.cmp(&y.block_number))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Url {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub urls_override: Option<HashMap<String, Url>>,
}

impl NodeConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        let setting_path = current_dir.join("config.yml");
        let s = Config::builder()
            .add_source(File::from(setting_path))
            .add_source(
                Environment::with_prefix("")
                    .prefix_separator("")
                    .separator("_"),
            )
            .build()?;
        s.try_deserialize()
    }
}
