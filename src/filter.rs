use crate::{res::HyperResponse, store, Request, Server};
use futures::future;
use warp::{filters, reject, Filter, Rejection};

/// Create a `Filter` that initialize JSON RPC handling.
pub fn json_rpc(server: Server) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    filters::method::post()
        .and(filters::header::exact("Content-Type", "application/json"))
        // Get and set `Request` if it is not stored already.
        .and(store::filled().or(store_req()))
}

fn store_req() -> impl Filter<Extract = (Request,), Error = Rejection> {
    filters::body::json::<Request>()
        .and(store::lazy_store())
        .map(|req, store| {
            store.fill(req.clone());
            req
        })
}

/// Create a `Filter` that requires the request RPC method to be given name.
pub fn method(name: &'static str) -> impl Filter<Extract = (), Error = Rejection> {
    store::stored_req().and_then(|req| {
        if req.method() == name {
            future::ok(())
        } else {
            future::err(reject::reject())
        }
    })
}

/// Create a `Filter` that extracts RPC parameter.
pub fn params<T>() -> impl Filter<Extract = (T,), Error = Rejection>
where
    for<'de> T: Deserialize<'de>,
{
    store::stored_req()
        .and_then(|req| future::ready(req.deserialize_param::<T>().map_err(|_| reject::reject())))
}
