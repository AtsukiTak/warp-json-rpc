use serde::{Deserialize, Serialize};

pub trait Method {
    /// TODO : Should be JSON-RPC 2.0 compatible deserialize spec
    type Params: Deserialize;
    type Response: Serialize;
    type Error: Serialize;

    const fn name() -> &'static str;

    fn call(&self, params: Self::Params) -> Result<Self::Response, Self::Error>;
}
