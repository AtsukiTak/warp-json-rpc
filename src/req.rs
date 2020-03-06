use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/*
 * =======
 * Request
 * =======
 */
#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub jsonrpc: Version,
    pub id: Option<u64>,
    pub method: String,
    pub params: Option<Box<RawValue>>,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

impl Request {
    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    pub fn deserialize_param<'de, T>(&'de self) -> Result<T, anyhow::Error>
    where
        T: Deserialize<'de>,
    {
        match &self.params {
            Some(params) => Ok(serde_json::from_str(params.get())?),
            None => Err(anyhow::anyhow!("No parameter is presented")),
        }
    }
}

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

        #[derive(PartialEq, Eq, Debug)]
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
        assert_eq!(req.id, Some(42));
        assert_eq!(req.method, "op".to_string());

        let (lhs, rhs, op) = req.deserialize_param::<(i32, i32, String)>().unwrap();
        assert_eq!(lhs, 24);
        assert_eq!(rhs, 12);
        assert_eq!(op, "+".to_string());
    }
}
