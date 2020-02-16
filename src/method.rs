use crate::{
    req::Request,
    res::{Error, Response},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::future::Future;

pub trait Method {
    /// TODO : Should be JSON-RPC 2.0 compatible deserialize spec
    /// If this method does not accept any parameters,
    /// you SHOULD set `Params = ()`;
    type Params: DeserializeOwned;
    type Response: Serialize;
    type ResponseFut: Future<Output = Result<Self::Response, Error>>;

    fn name() -> &'static str;

    fn new() -> Self;

    fn call(&self, params: Self::Params) -> Self::ResponseFut;
}

/*
pub enum MethodSet {
    Hoge(HogeMethod),
    Fuga(FugaMethod),
}

static METHOD_MAP: phf::Map<&'static str, MethodSet> = phf_map! {
    "hoge" => MethodSet::Hoge(HogeMethod::new())
    "fuga" => MethodSet::Fuga(FugaMethod::new())
}
*/

async fn call_method<M>(method: &M, req: Request) -> Response
where
    M: Method,
{
    call_method_inner(method, req).await.unwrap_or_else(|e| e)
}

async fn call_method_inner<M>(method: &M, req: Request) -> Result<Response, Response>
where
    M: Method,
{
    let id = req.id;

    // パラメータをパースする
    let raw_params = req.params.map(|p| p.into()).unwrap_or(Value::Null);
    let params = serde_json::from_value::<M::Params>(raw_params).map_err(|e| {
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
