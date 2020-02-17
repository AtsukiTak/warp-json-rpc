use crate::{
    req::Request,
    res::{Error, Response},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{future::Future, marker::PhantomData, pin::Pin};

/// If this method does not accept any parameters,
/// you SHOULD set `Params = ()`;
pub trait Method<Params: DeserializeOwned, ParamType> {
    type Response: Serialize;
    type ResponseFut: Future<Output = Result<Self::Response, Error>>;

    fn call(self, params: Params) -> Self::ResponseFut;
}

pub struct ArrayType();
pub struct MapType();

pub struct MethodFn<F, ParamType> {
    f: F,
    _param_type: PhantomData<ParamType>,
}

pub fn map_method_fn<F>(f: F) -> MethodFn<F, MapType> {
    MethodFn {
        f,
        _param_type: PhantomData,
    }
}

pub fn array_method_fn<F>(f: F) -> MethodFn<F, ArrayType> {
    MethodFn {
        f,
        _param_type: PhantomData,
    }
}

impl<F, P, S, R> Method<P, MapType> for MethodFn<F, MapType>
where
    F: FnOnce(P) -> S,
    P: DeserializeOwned,
    S: Future<Output = Result<R, Error>>,
    R: Serialize,
{
    type Response = R;
    type ResponseFut = S;

    fn call(self, params: P) -> S {
        (self.f)(params)
    }
}

impl<F, S, R> Method<(), ArrayType> for MethodFn<F, ArrayType>
where
    F: FnOnce() -> S,
    S: Future<Output = Result<R, Error>>,
    R: Serialize,
{
    type Response = R;
    type ResponseFut = S;

    fn call(self, _params: ()) -> S {
        (self.f)()
    }
}

impl<F, P, S, R> Method<(P,), ArrayType> for MethodFn<F, ArrayType>
where
    F: FnOnce(P) -> S,
    P: DeserializeOwned,
    S: Future<Output = Result<R, Error>>,
    R: Serialize,
{
    type Response = R;
    type ResponseFut = S;

    fn call(self, params: (P,)) -> S {
        (self.f)(params.0)
    }
}

impl<F, P1, P2, S, R> Method<(P1, P2), ArrayType> for MethodFn<F, ArrayType>
where
    F: FnOnce(P1, P2) -> S,
    P1: DeserializeOwned,
    P2: DeserializeOwned,
    S: Future<Output = Result<R, Error>>,
    R: Serialize,
{
    type Response = R;
    type ResponseFut = S;

    fn call(self, params: (P1, P2)) -> S {
        (self.f)(params.0, params.1)
    }
}

type RequestHandler = Box<dyn FnOnce(Request) -> Pin<Box<dyn Future<Output = Response>>>>;

/// `Method` を Request -> Response のクロージャにラップする
fn method_to_handler<M, P, T>(method: M) -> RequestHandler
where
    M: Method<P, T> + 'static,
    P: DeserializeOwned,
{
    // Request handler の本体
    async fn inner<M, P, T>(method: M, req: Request) -> Result<Response, Response>
    where
        M: Method<P, T>,
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

/// ```
/// {
///     pub enum MethodType {
///         hoge,
///         fuga,
///     }
///
///     fn get_method_type(name: &str) -> Option<MethodType> {
///         static METHOD_MAP: phf::Map<&'static str, MethodType> = phf_map! {
///             "hoge" => MethodType::hoge,
///             "fuga" => MethodType::fuga
///         };
///
///         METHOD_MAP.get(name).copied()
///     }
///
///     pub struct JsonRpcServer {
///         hoge_factory: RequestHandlerFactory,
///         fuga_factory: RequestHandlerFactory,
///     }
///
///     impl JsonRpcServer {
///         async fn handle_request(&self, req: Request) -> Response {
///             match get_method_type(req.method.as_str()) {
///                 None => Response::new_err(req.id, Error::METHOD_NOT_FOUND),
///                 Some(MethodType::hoge) => {
///                     let handler = (self.hoge_factory)();
///                     handler(req)
///                 },
///                 Some(MethodType::Fuga) => {
///                     let handler = (self.fuga_factory)();
///                     handler(req)
///                 }
///             }
///         }
///     }
///
///     JsonRpcServer {
///         hoge: gen_request_handler_factory(),
///         fuga: gen_request_handler_factory(),
///     }
/// }
/// ```
macro_rules! json_rpc {
    ( $( $key:ident => $factory:expr ),+ ) => {{
        #[allow(non_camel_casel_types)]
        #[derive(Clone, Copy)]
        enum MethodType {
            $( $key, )+
        }

        pub struct JsonRpcServer {
            $(
                $key: RequestHandlerFactory,
            )+
        }

        impl JsonRpcServer {
            fn get_method_type(name: &str) -> Option<MethodType> {
                static MAP: phf::Map<&'static str, MethodType> = phf::phf_map! {
                    $( stringify!($key) => MethodType::$key, )+
                };

                MAP.get(name).copied()
            }

            async fn handle_request(&self, req: Request) -> Response {
                match Self::get_method_type(req.method.as_str()) {
                    None => Response::new_err(req.id, Error::METHOD_NOT_FOUND),
                    $(
                        Some(MethodType::$key) => {
                            let handler = (self.$key)();
                            handler(req).await
                        },
                    )+
                }
            }
        }

        JsonRpcServer {
            $(
                $key: gen_request_handler_factory($factory),
            )+
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::res::ResponseContent;
    use serde_json::json;

    #[tokio::test]
    async fn macro_test_inner() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "add",
            "params": [1, 2],
            "id": 42
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();

        let add_method_factory = || method_fn(|(a, b): (i32, i32)| async move { Ok(a + b) });
        let server = json_rpc! {
            add => add_method_factory
        };

        let res = server.handle_request(req).await;
        assert_eq!(res.id, Some(42));
        assert_eq!(res.content, ResponseContent::Success(json!(3)));
    }
}
