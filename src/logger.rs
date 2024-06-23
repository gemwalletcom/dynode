use hyper::{body::Incoming as IncomingBody, header, Request, Response};

use crate::request_url::RequestUrl;

pub fn log_incoming_request(request: &Request<IncomingBody>) {
    let headers = request.headers().clone();
    let user_agent = headers.get(header::USER_AGENT);
    let host = headers
        .get("host")
        .expect("invalid host")
        .to_str()
        .unwrap_or_default();

    println!(
        "incoming request: {} {} {} {:?}",
        host,
        request.method(),
        request.uri(),
        user_agent
    );
}

pub fn log_proxy_response(request: &RequestUrl, response: &Response<IncomingBody>) {
    println!(
        "proxy response: {:?} {}",
        request.uri.host().unwrap_or_default(),
        response.status()
    );
}
