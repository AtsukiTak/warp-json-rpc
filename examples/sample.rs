use futures::future;
use warp::Filter as _;
use warp_json_rpc::{array_method_fn, json_rpc, Error, Server};

async fn add(a: usize, b: usize) -> Result<usize, Error> {
    Ok(a + b)
}

#[tokio::main]
async fn main() {
    // Plain function method
    let add_method_factory = || array_method_fn(add);

    // Closure function method
    let prefix = "Hello ";
    let greet_method_factory =
        move || array_method_fn(move |name: String| future::ok(format!("{}{}", prefix, name)));

    // Create Json RPC server
    let server = Server::builder()
        .register("add", add_method_factory)
        .register("greet", greet_method_factory)
        .build();

    let log = warp::filters::log::custom(|info| {
        eprintln!("{} {} {}", info.method(), info.path(), info.status());
    });
    let filter = warp::filters::path::path("api")
        .and(json_rpc(server))
        .with(log);

    warp::serve(filter).bind(([127, 0, 0, 1], 8842)).await
}
