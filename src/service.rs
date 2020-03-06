use crate::store::LazyReqStore;
use core::{
    convert::Infallible,
    task::{Context, Poll},
};
use futures::future::{Future, TryFuture};
use http::Request;
use hyper::{service::Service, Body};
use warp::{
    reply::{Reply, Response},
    Filter, Rejection,
};

pub struct JsonRpcService<S> {
    service: S,
}

impl<S> Service<Request<Body>> for JsonRpcService<S>
where
    S: Service<Request<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        // Add `LazyReqStore` if it does not exist
        let ext = req.extensions_mut();
        if ext.get::<LazyReqStore>().is_none() {
            ext.insert(LazyReqStore::empty());
        }

        self.service.call(req)
    }
}

impl<S> JsonRpcService<S> {
    pub fn new(service: S) -> JsonRpcService<S> {
        JsonRpcService { service }
    }
}

pub fn service<F>(
    filter: F,
) -> impl Service<
    Request<Body>,
    Response = Response,
    Error = Infallible,
    Future = impl Future<Output = Result<Response, Infallible>>,
>
where
    F: Filter,
    F::Future: TryFuture<Error = Rejection>,
    <F::Future as TryFuture>::Ok: Reply,
{
    JsonRpcService::new(warp::service(filter))
}
