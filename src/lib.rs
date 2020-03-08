//! # Usage
//!
//! ```no_run
//! use warp_json_rpc::{filters as json_rpc, Builder};
//! use futures::future;
//! use warp::Filter as _;
//! use std::convert::Infallible;
//!
//! #[tokio::main]
//! async fn main() {
//!   // create Filter
//!   let route = warp::filters::path::path("rpc")
//!     // ## Point 1
//!     // This filter is required.
//!     .and(json_rpc::json_rpc())
//!     .and(json_rpc::method("add"))
//!     .and(json_rpc::params::<(usize, usize)>())
//!     // `res.success` returns `impl Reply` which represents JSON RPC Response
//!     .map(|res: Builder, (lhs, rhs)| res.success(lhs + rhs).unwrap());
//!
//!   // ## Point 2
//!   // You **MUST** wraps root `Filter` by `warp_json_rpc::service` function.
//!   let svc = warp_json_rpc::service(route);
//!   let make_svc = hyper::service::make_service_fn(move |_| future::ok::<_, Infallible>(svc));   
//!   hyper::Server::bind(&([127, 0, 0, 1], 3030).into())
//!     .serve(make_svc)
//!     .await
//!     .unwrap();
//! }
//! ```
pub mod filters;
mod req;
mod res;
mod service;
mod store;

pub use req::Request;
pub use res::{Builder, Error};
pub use service::service;
