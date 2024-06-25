use std::collections::HashMap;

use crate::{
    chain_service::ChainService,
    chain_type::ChainType,
    config::Domain,
    proxy_request_service::{NodeDomain, ProxyRequestService},
};

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
}

impl NodeService {
    pub fn get_proxy_request(&self) -> ProxyRequestService {
        ProxyRequestService {
            domains: self.get_node_domains(),
        }
    }

    pub fn get_node_domains(&self) -> HashMap<String, NodeDomain> {
        self.domains
            .clone()
            .into_iter()
            .map(|(key, domain)| {
                let url = domain.urls.first().unwrap().clone();
                let mut urls_override = domain.urls_override.unwrap_or_default();
                urls_override.extend(url.clone().urls_override.unwrap_or_default().into_iter());

                (key, NodeDomain { url, urls_override })
            })
            .collect()
    }

    pub async fn update_block_numbers(&self) {
        for (_, domain) in self.domains.clone() {
            for url in domain.urls.clone() {
                let _ = self
                    .update_latest_block(domain.chain_type.clone(), url.url.as_str())
                    .await;
            }
        }
    }

    pub async fn update_latest_block(&self, chain_type: ChainType, url: &str) {
        let chain_service = ChainService {
            chain_type: chain_type.clone(),
            url: url.to_string(),
        };

        let res = chain_service.get_block_number().await;

        println!(
            "update_latest_block: chain_type: {:?}, url: {} {:?}",
            chain_type, url, res
        );
    }
}
