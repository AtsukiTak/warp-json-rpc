use serde::{Deserialize, Serialize};
use serde_json::{map::Map, Value};

/*
 * =======
 * Request
 * =======
 */
#[derive(Debug, Deserialize)]
pub struct Request {
    pub jsonrpc: Version,
    pub id: Option<u64>,
    pub method: String,
    pub params: Option<Params>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Deserialize)]
pub enum Params {
    ByPosition(Vec<Value>),
    ByName(Map<String, Value>),
}
