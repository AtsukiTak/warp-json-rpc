use crate::{
    method::{Method, MethodFactory},
    req::Request,
    res::{Error, Response},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

#[derive(Clone)]
pub struct Server {
    methods: Arc<HashMap<&'static str, RequestHandlerFactory>>,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    pub async fn handle_request(&self, req: Request) -> Response {
        match self.methods.get(req.method.as_str()) {
            None => Response::new_err(req.id, Error::METHOD_NOT_FOUND),
            Some(factory) => {
                let handler = factory();
                handler(req).await
            }
        }
    }
}

pub struct ServerBuilder {
    methods: HashMap<&'static str, RequestHandlerFactory>,
}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder {
            methods: HashMap::new(),
        }
    }

    pub fn register<F, M, P>(mut self, name: &'static str, method: F) -> Self
    where
        F: MethodFactory<M> + 'static + Send + Sync,
        M: Method<P> + 'static + Send + Sync,
        P: DeserializeOwned + 'static + Send + Sync,
    {
        self.methods
            .insert(name, method_factory_to_handler_factory(method));
        self
    }

    pub fn build(self) -> Server {
        Server {
            methods: Arc::new(self.methods),
        }
    }
}

type ResponseFut = Pin<Box<dyn Future<Output = Response> + Send + Sync>>;
type RequestHandler = Box<dyn FnOnce(Request) -> ResponseFut + Send + Sync>;
type RequestHandlerFactory = Box<dyn Fn() -> RequestHandler + Send + Sync>;

/// `Method` を Request -> Response のクロージャにラップする
fn method_to_handler<M, P>(method: M) -> RequestHandler
where
    M: Method<P> + 'static + Send + Sync,
    P: DeserializeOwned + 'static + Send + Sync,
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
    F: MethodFactory<M> + 'static + Send + Sync,
    M: Method<P> + 'static + Send + Sync,
    P: DeserializeOwned + 'static + Send + Sync,
{
    Box::new(move || method_to_handler(factory.create()))
}
