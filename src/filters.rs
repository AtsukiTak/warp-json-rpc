use crate::{
    store::{self, LazyReqStore},
    Builder, Request,
};
use futures::future;
use serde::Deserialize;
use warp::{filters, reject, Filter, Rejection};

/// Create a `Filter` that requires and initializes JSON RPC handling.
pub fn json_rpc() -> impl Filter<Extract = (Builder,), Error = Rejection> + Clone {
    filters::method::post()
        .and(filters::header::exact("Content-Type", "application/json"))
        // Get and set `Request` if it is not stored already.
        .and(store::filled().or(store_req()))
        .map(|_| ())
        .untuple_one()
        .and(store::stored_req().map(|req: Request| Builder::new(req.id)))
}

fn store_req() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    filters::body::json::<Request>()
        .and(store::store())
        .map(|req: Request, store: LazyReqStore| {
            store
                .fill(req.clone())
                .expect("LazyReqStore is filled more than twice");
        })
        .untuple_one()
}

/// Create a `Filter` that requires the request RPC method to be given name.
pub fn method(name: &'static str) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    store::stored_req()
        .and_then(move |req: Request| {
            if req.method() == name {
                future::ok(())
            } else {
                future::err(reject::reject())
            }
        })
        .untuple_one()
}

/// Create a `Filter` that extracts RPC parameter.
pub fn params<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    for<'de> T: Deserialize<'de> + Send,
{
    store::stored_req().and_then(|req: Request| {
        future::ready(req.deserialize_param::<T>().map_err(|_| reject::reject()))
    })
}
