use crate::{
    method::{Method, MethodFactory},
    req::Request,
    res::{Error, Response},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::HashMap, future::Future, pin::Pin};

pub struct Server {
    methods: HashMap<&'static str, RequestHandlerFactory>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            methods: HashMap::new(),
        }
    }

    pub fn register<F, M, P>(&mut self, name: &'static str, method: F)
    where
        F: MethodFactory<M> + 'static,
        M: Method<P> + 'static,
        P: DeserializeOwned,
    {
        self.methods
            .insert(name, method_factory_to_handler_factory(method));
    }
}

type RequestHandler = Box<dyn FnOnce(Request) -> Pin<Box<dyn Future<Output = Response>>>>;
type RequestHandlerFactory = Box<dyn FnOnce() -> RequestHandler>;

/// `Method` を Request -> Response のクロージャにラップする
fn method_to_handler<M, P>(method: M) -> RequestHandler
where
    M: Method<P> + 'static,
    P: DeserializeOwned,
{
    // Request handler の本体
    async fn inner<M, P>(method: M, req: Request) -> Result<Response, Response>
    where
        M: Method<P>,
        P: DeserializeOwned,
    {
        let id = req.id;

        // パラメータをパースする
        let raw_params = req.params.map(|p| p.into()).unwrap_or(Value::Null);
        let params = serde_json::from_value::<P>(raw_params).map_err(|e| {
            log::debug!("Failed to parse Json RPC params");
            log::debug!("   {:?}", e);
            Response::new_err(id, Error::INVALID_PARAMS)
        })?;

        // メソッドを呼び出す
        method
            .call(params)
            .await
            .map_err(|e| {
                log::debug!("Return error");
                log::debug!("   {:?}", e);
                Response::new_err(id, e)
            })
            .map(|res| Response::new(id, res))
    }

    Box::new(move |req| Box::pin(async { inner(method, req).await.unwrap_or_else(|e| e) }))
}

fn method_factory_to_handler_factory<F, M, P>(factory: F) -> RequestHandlerFactory
where
    F: MethodFactory<M> + 'static,
    M: Method<P> + 'static,
    P: DeserializeOwned,
{
    Box::new(move || method_to_handler(factory.create()))
}
