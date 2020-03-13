use crate::{
    store::{self, LazyReqStore},
    Builder, Request,
};
use futures::future;
use serde::Deserialize;
use warp::{filters, reject, Filter, Rejection};

/// Create a [`Filter`] that requires and initializes JSON RPC handling.
///
/// Note that you **MUST** call this [`Filter`] before [`method`] or [`params`] method.
///
/// [`Filter`]: https://docs.rs/warp/0.2.2/warp/trait.Filter.html
/// [`method`]: ./fn.method.html
/// [`params`]: ./fn.params.html
///
/// ```
/// # use warp_json_rpc::filters::*;
/// # use warp::Filter as _;
///
/// let rpc = json_rpc().and(method("greet")).and(params::<(String,)>());
/// ```
pub fn json_rpc() -> impl Filter<Extract = (Builder,), Error = Rejection> + Copy {
    filters::method::post()
        .and(filters::header::exact("Content-Type", "application/json"))
        // Get and set `Request` if it is not stored already.
        .and(store::filled().or(store_req()))
        .map(|_| ())
        .untuple_one()
        .and(store::stored_req().map(|req: Request| Builder::new(req.id())))
}

fn store_req() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    filters::body::json::<Request>()
        .and(store::store())
        .map(|req: Request, store: LazyReqStore| {
            store
                .fill(req)
                .expect("LazyReqStore is filled more than twice");
        })
        .untuple_one()
}

/// Create a `Filter` that requires the request RPC method to be given name.
///
/// Note that you **MUST** call [`json_rpc`] filter first.
///
/// [`json_rpc`]: ./fn.json_rpc.html
///
/// ```
/// # use warp_json_rpc::filters::*;
/// # use warp::Filter as _;
///
/// let rpc = json_rpc().and(method("greet"));
/// ```
pub fn method(name: &'static str) -> impl Filter<Extract = (), Error = Rejection> + Copy {
    store::stored_req()
        .and_then(move |req: Request| {
            if req.method() == name {
                log::info!(target: "warp_json_rpc", "\"{}\" RPC", name);
                future::ok(())
            } else {
                future::err(reject::reject())
            }
        })
        .untuple_one()
}

/// Create a `Filter` that extracts RPC parameter.
///
/// Note that you **MUST** call [`json_rpc`] filter first.
///
/// [`json_rpc`]: ./fn.json_rpc.html
///
/// ```
/// # use warp_json_rpc::filters::*;
/// # use warp::Filter as _;
///
/// let rpc = json_rpc().and(method("greet")).and(params::<(String,)>());
/// ```
pub fn params<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy
where
    for<'de> T: Deserialize<'de> + Send,
{
    store::stored_req().and_then(|req: Request| {
        future::ready(req.deserialize_param::<T>().map_err(|_| reject::reject()))
    })
}
