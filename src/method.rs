use crate::{
    req::Request,
    res::{Error, Response},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{future::Future, marker::PhantomData, pin::Pin};

/// If this method does not accept any parameters,
/// you SHOULD set `Params = ()`;
pub trait Method<Params: DeserializeOwned> {
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

impl<F, P, S, R> Method<P> for MethodFn<F, MapType>
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

impl<F, S, R> Method<()> for MethodFn<F, ArrayType>
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

macro_rules! array_method_fns {
    ($t:ident) => {
        impl<F, $t, S, R> Method<($t,)> for MethodFn<F, ArrayType>
        where
            F: FnOnce($t) -> S,
            $t: DeserializeOwned,
            S: Future<Output = Result<R, Error>>,
            R: Serialize,
        {
            type Response = R;
            type ResponseFut = S;

            fn call(self, params: ($t,)) -> S {
                (self.f)(params.0)
            }
        }
    };

    ($t1:ident, $( $t:ident ),* ) => {
        array_method_fns!( $( $t ),* );

        impl<F, $t1, $( $t ),*, S, R> Method<($t1, $( $t ),*)> for MethodFn<F, ArrayType>
        where
            F: FnOnce($t1, $( $t ),*) -> S,
            $t1: DeserializeOwned,
            $( $t: DeserializeOwned, )*
            S: Future<Output = Result<R, Error>>,
            R: Serialize,
        {
            type Response = R;
            type ResponseFut = S;

            fn call(self, params: ($t1, $( $t ),*)) -> S {
                #[allow(non_snake_case)]
                let ($t1, $( $t ),*) = params;
                (self.f)($t1, $( $t ),*)
            }
        }
    };
}

array_method_fns! {
    T16,
    T15,
    T14,
    T13,
    T12,
    T11,
    T10,
    T9,
    T8,
    T7,
    T6,
    T5,
    T4,
    T3,
    T2,
    T1
}

type RequestHandler = Box<dyn FnOnce(Request) -> Pin<Box<dyn Future<Output = Response>>>>;

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
