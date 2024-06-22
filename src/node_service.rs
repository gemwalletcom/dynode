use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{self, HeaderName, USER_AGENT};
use hyper::service::Service;
use hyper::HeaderMap;

use futures::FutureExt;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::config::Domain;

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
}

impl Service<Request<IncomingBody>> for NodeService {
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let host = req.headers().get("host").unwrap().to_str().unwrap();
        //let USER_AGENT =
        //println!("host: {}, user-agent: {}", host, req.headers().get("key"));

        match self.domains.get(host) {
            Some(domain) => {
                let domain_url = domain.urls.first().unwrap().url.clone();
                let uri = domain_url + req.uri().to_string().as_str();
                let uri = uri.parse::<hyper::Uri>().unwrap();

                async move { proxy_pass(req, uri).await }.boxed()
            }
            _ => async move { unsupported_chain(req).await }.boxed(), //async move { handle_request(req).await }.boxed(), //Ok(Response::new(Full::from("unsupported domain")))},
        }
    }
}

async fn unsupported_chain(
    _req: Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    Err("Unsupported chain: {}".into())
}

async fn proxy_pass(
    original_request: Request<IncomingBody>,
    uri: hyper::Uri,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpsConnector::new());

    // request
    let original_headers = original_request.headers().clone();
    let mut request = Request::builder()
        .method(original_request.method())
        .uri(uri)
        .body(original_request.into_body())
        .expect("invalid request params");

    *request.headers_mut() = persist_headers(&original_headers, &vec![header::CONTENT_TYPE]);

    // response
    let response = client.request(request).await?;
    let resp_headers = response.headers().clone();
    let body = response.collect().await?.to_bytes();

    let mut new_response = Response::new(Full::from(body));
    *new_response.headers_mut() = persist_headers(&resp_headers, &vec![header::CONTENT_TYPE]);

    Ok(new_response)
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
