#![allow(unused)]

use std::{collections::HashMap, env, hash::Hasher};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use crate::chain_type::ChainType;

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
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
pub struct Domain {
    pub domain: String,
    pub chain_type: ChainType,
    pub block_delay: Option<u64>,
    pub urls: Vec<Url>,
    pub urls_override: Option<HashMap<String, Url>>,
}

#[derive(Debug, Deserialize, Clone)]
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
