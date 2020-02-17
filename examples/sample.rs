use warp_json_rpc::{array_method_fn, Error, Server};

async fn add(a: usize, b: usize) -> Result<usize, Error> {
    Ok(a + b)
}

fn main() {
    let a = 42;
    let add_method_factory = move || array_method_fn(move |b| add(a, b));

    let mut server = Server::new();
    server.register("add_42", add_method_factory);
}
