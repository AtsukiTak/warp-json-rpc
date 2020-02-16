use serde::{Deserialize, Serialize};
use serde_json::{map::Map, Value};
use warp::Filter as _;

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
#[serde(untagged)]
pub enum Params {
    ByPosition(Vec<Value>),
    ByName(Map<String, Value>),
}

impl Into<Value> for Params {
    fn into(self) -> Value {
        match self {
            Params::ByPosition(inner) => Value::Array(inner),
            Params::ByName(inner) => Value::Object(inner),
        }
    }
}

/*
pub fn request() -> impl warp::Filter<Extract = (Request,), Error = Rejection> {
    warp::filters::method::post()
        .and(warp::filters::header::exact(
            "Content-Type",
            "application/json",
        ))
        .and(warp::filters::body::json::<Request>())
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_by_name_request() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "params": { "lhs": 24, "rhs": 12, "op": "+"},
            "id": 42
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();
        assert_eq!(req.id, Some(42));
        assert_eq!(req.method, "op".to_string());

        match req.params.unwrap() {
            Params::ByName(map) => {
                assert_eq!(map.get("lhs"), Some(&json!(24)));
                assert_eq!(map.get("rhs"), Some(&json!(12)));
                assert_eq!(map.get("op"), Some(&json!("+")));
            }
            Params::ByPosition(_) => {
                panic!("Error: parameter should be ByName");
            }
        }
    }

    #[test]
    fn deserialize_by_pos_request() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "params": [24, 12, "+"],
            "id": 42
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();
        assert_eq!(req.id, Some(42));
        assert_eq!(req.method, "op".to_string());

        match req.params.unwrap() {
            Params::ByPosition(arr) => {
                assert_eq!(arr[0], json!(24));
                assert_eq!(arr[1], json!(12));
                assert_eq!(arr[2], json!("+"));
            }
            Params::ByName(_) => {
                panic!("Error: parameter should be ByPosition");
            }
        }
    }
}
