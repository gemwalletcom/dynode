use hyper::{body::Incoming as IncomingBody, header, Request};

pub fn log_incoming_request(request: &Request<IncomingBody>) {
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
