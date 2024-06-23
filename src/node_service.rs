use std::collections::HashMap;

use crate::{
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
}
