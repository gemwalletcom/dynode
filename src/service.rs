use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::service::Service;
use hyper::HeaderMap;

use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use futures::FutureExt;
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

        match self.domains.get(host) {
            Some(domain) => {
                println!("test : {:?}", domain);

                let url = domain.urls.first().unwrap().url.clone();
                let uri = url.parse::<hyper::Uri>().unwrap();

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
    let method = original_request.method();

    let original_headers = original_request.headers().clone();

    //TODO: Persist only needed headers
    let mut new_headers = HeaderMap::new();
    new_headers.append(
        "Content-Type",
        original_headers.get("Content-Type").unwrap().clone(),
    );

    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .body(original_request.into_body())
        .expect("msg");

    *request.headers_mut() = new_headers;

    let resp = client.request(request).await?;
    let body = resp.collect().await?.to_bytes();
    Ok(Response::new(Full::from(body)))
}
