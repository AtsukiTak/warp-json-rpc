use serde::Deserialize;
use serde_json::{map::Map, Value};

#[derive(Debug, Deserialize)]
pub struct Request {
    jsonrpc: Version,
    id: Option<u64>,
    method: String,
    params: Option<Params>,
}

#[derive(Debug, Deserialize)]
enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Deserialize)]
pub enum Params {
    ByPosition(Vec<Value>),
    ByName(Map<String, Value>),
}
