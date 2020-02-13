use crate::{method::Method, response::Response as JsonRpcResponse};
use hyper::{Request, Response};
use std::collections::HashMap;

pub struct Server {
    methods: HashMap<&'static str, Method>,
}

impl Service<Request<Body>> for Server {
    type Response = Response<Body>;
    type Error = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Vec<u8>>) -> Self::Future {
        if *req.method() != Method::POST {
            let res = JsonRpcResponse::new_err(None, ErrorObject::INVALID_REQUEST).into();
            return Box::pin(future::ok(res));
        }
        req.body
    }
}
