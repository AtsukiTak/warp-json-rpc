use crate::{res::HyperResponse, Request, Server};
use futures::future;
use warp::{filters, Filter, Rejection};

/// Create a `Filter` that initialize JSON RPC handling.
pub fn json_rpc(server: Server) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    filters::method::post()
        .and(filters::header::exact("Content-Type", "application/json"))
        // Get and set `Request` if it is not stored already.
        .and(store::filled().or(store_req()))
}

fn store_req() -> impl Filter<Extract = (Request,), Error = Rejection> {
    filters::body::json::<Request>()
        .and(lazy_store())
        .map(|req, store| {
            store.fill(req.clone());
            req
        })
}
