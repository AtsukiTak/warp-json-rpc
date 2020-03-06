use core::convert::Infallible;
use futures::future;
use warp::Filter as _;
use warp_json_rpc::Builder;

#[tokio::main]
async fn main() {
    let route = warp::filters::path::path("rpc")
        .and(warp_json_rpc::filters::json_rpc())
        .and(warp_json_rpc::filters::method("add"))
        .and(warp_json_rpc::filters::params::<(isize, isize)>())
        .map(|res: Builder, (lhs, rhs)| res.success(lhs + rhs).unwrap());

    let svc = warp_json_rpc::service(route);
    let make_svc = hyper::service::make_service_fn(move |_| future::ok::<_, Infallible>(svc));
    hyper::Server::bind(&([127, 0, 0, 1], 3030).into())
        .serve(make_svc)
        .await
        .unwrap();
}
