use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Arc;

/*
 * =======
 * Request
 * =======
 */
/// Represents each JSON RPC request.
// Deserializing structs containing flattened RawValue always fails.
// https://github.com/serde-rs/json/issues/599
//
// So currently we wrap `method` and `params` by `Arc` separately.
#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    jsonrpc: Version,
    id: Id,
    method: Arc<String>,
    params: Arc<Option<Box<RawValue>>>,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    String(Arc<String>),
    Number(i64),
    Null,
}

impl From<&str> for Id {
    fn from(id: &str) -> Self {
        Id::String(Arc::new(id.to_string()))
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Id::String(Arc::new(id))
    }
}

impl From<i64> for Id {
    fn from(id: i64) -> Self {
        Id::Number(id)
    }
}

impl Request {
    pub fn id(&self) -> Id {
        self.id.clone()
    }

    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    pub fn deserialize_param<'de, T>(&'de self) -> Result<T, anyhow::Error>
    where
        T: Deserialize<'de>,
    {
        match self.params.as_ref() {
            Some(params) => Ok(serde_json::from_str(params.get())?),
            None => Err(anyhow::anyhow!("No parameter is presented")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_by_name_request() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "params": { "lhs": 24, "rhs": 12, "op": "+"},
            "id": 42
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();
        assert_eq!(req.id(), Id::Number(42));
        assert_eq!(req.method(), "op");

        #[derive(PartialEq, Eq, Debug, Deserialize)]
        struct Param {
            lhs: i32,
            rhs: i32,
            op: String,
        }

        let param = req.deserialize_param::<Param>().unwrap();
        assert_eq!(
            param,
            Param {
                lhs: 24,
                rhs: 12,
                op: "+".to_string()
            }
        );
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
        assert_eq!(req.id(), Id::Number(42));
        assert_eq!(req.method(), "op");

        let (lhs, rhs, op) = req.deserialize_param::<(i32, i32, String)>().unwrap();
        assert_eq!(lhs, 24);
        assert_eq!(rhs, 12);
        assert_eq!(op, "+".to_string());
    }

    #[test]
    fn deserialize_string_id() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "id": "string identifier"
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();
        assert_eq!(req.id(), Id::from("string identifier"));
        assert_eq!(req.id(), Id::from(String::from("string identifier")));
    }

    #[test]
    fn deserialize_number_id() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "id": -1234
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();
        assert_eq!(req.id(), Id::from(-1234));
    }

    #[test]
    fn deserialize_null_id() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "id": null
        }"#;
        let req = serde_json::from_str::<Request>(req_str).unwrap();
        assert_eq!(req.id(), Id::Null);
    }

    #[test]
    fn deserialize_no_id_should_fail() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op"
        }"#;
        assert!(serde_json::from_str::<Request>(req_str).is_err());
    }

    #[test]
    fn deserialize_bad_id() {
        let req_str = r#"{
            "jsonrpc": "2.0",
            "method": "op",
            "id": 1.0
        }"#;
        assert!(serde_json::from_str::<Request>(req_str).is_err());
    }
}
