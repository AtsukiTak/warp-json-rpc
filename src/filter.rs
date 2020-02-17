use crate::{Request, Response, Server};
use warp::{filters, Filter};

pub fn json_rpc(server: Server) -> impl Filter<Extract = (Response,)> {
    filters::method::post()
        .and(filters::body::json())
        .and(warp::filters::header::exact(
            "Content-Type",
            "application/json",
        ))
        .and_then(move |req: Request| async move {
            let res = server.handle_request(req).await;
            Ok(res)
        })
}
