use std::pin::Pin;

use bytes::Bytes;
use futures::Future;
use http_body_util::Full;
use hyper::{body::Incoming as IncomingBody, service::Service, Request, Response};

#[derive(Debug, Clone)]
pub struct MetricsService {}

impl Service<Request<IncomingBody>> for MetricsService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        let res = mk_response("oh no! not found".into());

        Box::pin(async { res })
    }
}
