use crate::{res::HyperResponse, Request, Server};
use warp::{filters, Filter, Rejection};

pub fn json_rpc(server: Server) -> impl Filter<Extract = (HyperResponse,)> {
    filters::method::post()
        .and(warp::filters::header::exact(
            "Content-Type",
            "application/json",
        ))
        .and(filters::body::json())
        .and_then(move |req: Request| {
            let server2 = server.clone();
            async move {
                let res = server2.handle_request(req).await;
                let hyper_res = (&res).into();
                Result::<_, Rejection>::Ok(hyper_res)
            }
        })
}
