use crate::{Request, Server};
use warp::{filters, Filter as _};

pub fn json_rpc(server: Server) -> impl Filter<Extract = Server> {
    filters::method::post()
        .and(filters::body::json())
        .and(warp::filters::header::exact(
            "Content-Type",
            "application/json",
        ))
        .map(|req: Request| server.handle_request(req))
}
