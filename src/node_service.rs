use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{self, HeaderName};
use hyper::service::Service;
use hyper::{HeaderMap, Uri};

use futures::FutureExt;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use crate::config::{Domain, Url};

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
}

#[derive(Debug, Clone)]
pub struct RequestUrl {
    pub uri: Uri,
    pub params: HashMap<String, String>,
}

impl Service<Request<IncomingBody>> for NodeService {
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

        log_request(&req);

        match self.domains.get(host) {
            Some(domain) => {
                let url = domain.urls.first().unwrap().clone();

                let url = uri(url, req.uri());

                async move { proxy_pass(req, url).await }.boxed()
            }
            _ => async move { unsupported_chain(req).await }.boxed(), //async move { handle_request(req).await }.boxed(), //Ok(Response::new(Full::from("unsupported domain")))},
        }
    }
}

fn uri(url: Url, original_uri: &Uri) -> RequestUrl {
    let uri = url.url + original_uri.to_string().as_str();
    let uri = uri.parse::<hyper::Uri>().expect("invalid url");

    for (path, endpoint) in url.override_urls.unwrap_or_default().clone() {
        if uri.path() == path {
            let uri = Uri::from_str(&endpoint.url.clone().as_str()).unwrap();
            return RequestUrl {
                uri,
                params: endpoint.headers.unwrap_or_default(),
            };
        }
    }

    RequestUrl {
        uri,
        params: url.headers.unwrap_or_default(),
    }
}

fn log_request(request: &Request<IncomingBody>) {
    let headers = request.headers().clone();
    let user_agent = headers.get(header::USER_AGENT);
    let host = headers
        .get("host")
        .expect("invalid host")
        .to_str()
        .unwrap_or_default();

    println!(
        "{} {} {} {:?}",
        host,
        request.method(),
        request.uri(),
        user_agent
    );
}

async fn unsupported_chain(
    _req: Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    Err("Unsupported chain: {}".into())
}

async fn proxy_pass(
    original_request: Request<IncomingBody>,
    url: RequestUrl,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpsConnector::new());

    let keep_headers = vec![header::CONTENT_TYPE, header::CONTENT_ENCODING];

    // request
    let original_headers = original_request.headers().clone();
    let mut request = Request::builder()
        .method(original_request.method())
        .uri(url.uri)
        .body(original_request.into_body())
        .expect("invalid request params");

    // append url params
    let mut new_headers = persist_headers(&original_headers, &keep_headers);
    for (key, value) in url.params.clone() {
        new_headers.append(
            HeaderName::from_str(&key).unwrap(),
            value.clone().parse().unwrap(),
        );
    }
    *request.headers_mut() = new_headers;

    // response
    let response = client.request(request).await?;
    let resp_headers = response.headers().clone();
    let body = response.collect().await?.to_bytes();

    let mut new_response = Response::new(Full::from(body));
    *new_response.headers_mut() = persist_headers(&resp_headers, &keep_headers);

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
