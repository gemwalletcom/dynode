use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{self, HeaderName};
use hyper::service::Service;
use hyper::HeaderMap;

use futures::FutureExt;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Instant;

use crate::config::Url;
use crate::logger::{log_incoming_request, log_proxy_response};
use crate::metrics::Metrics;
use crate::request_url::RequestUrl;

#[derive(Debug, Clone)]
pub struct ProxyRequestService {
    pub domains: HashMap<String, NodeDomain>,
    pub metrics: Metrics,
}

#[derive(Debug, Clone)]
pub struct NodeDomain {
    pub url: Url,
}

impl Service<Request<IncomingBody>> for ProxyRequestService {
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let host = req
            .headers()
            .get("host")
            .expect("invalid host")
            .to_str()
            .unwrap_or_default();

        log_incoming_request(&req);

        match self.domains.get(host) {
            Some(domain) => {
                let url = domain.url.clone();
                let url = RequestUrl::from_uri(
                    url.clone(),
                    url.urls_override.clone().unwrap_or_default(),
                    req.uri(),
                );

                self.metrics.add_proxy_request(host);

                let metrics = self.metrics.clone();
                let host = host.to_string();

                async move {
                    let now = Instant::now();

                    let response = Self::proxy_pass_get_data(req, url.clone()).await?;

                    log_proxy_response(&url, &response, now.elapsed().as_millis());

                    metrics.add_proxy_response(
                        host.as_str(),
                        url.uri.path(),
                        url.uri.host().unwrap_or_default(),
                        response.status().as_u16(),
                        now.elapsed().as_millis(),
                    );

                    Self::proxy_pass_response(response).await
                }
                .boxed()
            }
            _ => async move {
                Ok(Response::builder()
                    .body(Full::new(Bytes::from("unsupported domain")))
                    .unwrap())
            }
            .boxed(),
        }
    }
}

impl ProxyRequestService {
    async fn proxy_pass_response(
        response: Response<IncomingBody>,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let keep_headers = vec![header::CONTENT_TYPE, header::CONTENT_ENCODING];

        let resp_headers = response.headers().clone();
        let status =  response.status();
        let body = response.collect().await?.to_bytes();

        let mut new_response = Response::new(Full::from(body));
        *new_response.status_mut() = status;
        *new_response.headers_mut() = Self::persist_headers(&resp_headers, &keep_headers);

        Ok(new_response)
    }

    async fn proxy_pass_get_data(
        original_request: Request<IncomingBody>,
        url: RequestUrl,
    ) -> Result<Response<IncomingBody>, Box<dyn std::error::Error + Send + Sync>> {
        let client =
            Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpsConnector::new());

        let keep_headers = vec![header::CONTENT_TYPE, header::CONTENT_ENCODING];

        // request
        let original_headers = original_request.headers().clone();
        let mut request = Request::builder()
            .method(original_request.method())
            .uri(url.clone().uri)
            .body(original_request.into_body())
            .expect("invalid request params");

        // append url params
        let mut new_headers = Self::persist_headers(&original_headers, &keep_headers);
        for (key, value) in url.params.clone() {
            new_headers.append(
                HeaderName::from_str(&key).unwrap(),
                value.clone().parse().unwrap(),
            );
        }
        *request.headers_mut() = new_headers;

        Ok(client.request(request).await?)
    }

    pub fn persist_headers(headers: &HeaderMap, list: &[HeaderName]) -> HeaderMap {
        headers
            .iter()
            .filter_map(|(k, v)| {
                if list.contains(&k) {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}
