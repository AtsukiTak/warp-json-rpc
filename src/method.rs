use crate::res::Error;
use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, marker::PhantomData};

/// If this method does not accept any parameters,
/// you SHOULD set `Params = ()`;
pub trait Method<Params: DeserializeOwned> {
    type Response: Serialize;
    type ResponseFut: Future<Output = Result<Self::Response, Error>>;

    fn call(self, params: Params) -> Self::ResponseFut;
}

pub enum ArrayType {}
pub enum MapType {}

/// Closureから生成された `Method`
pub struct MethodFn<F, ParamType> {
    f: F,
    _param_type: PhantomData<ParamType>,
}

/// パラメータをByNameで受け取るMethodを生成する
pub fn map_method_fn<F>(f: F) -> MethodFn<F, MapType> {
    MethodFn {
        f,
        _param_type: PhantomData,
    }
}

/// パラメータをByPositionで受け取るMethodを生成する
pub fn array_method_fn<F>(f: F) -> MethodFn<F, ArrayType> {
    MethodFn {
        f,
        _param_type: PhantomData,
    }
}

// 実装

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
