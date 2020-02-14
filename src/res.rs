use crate::req::Version;
use hyper::{Body, Response as HyperResponse};
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;

/*
 * ========
 * Response
 * ========
 */
#[derive(Debug, Serialize)]
pub struct Response {
    jsonrpc: Version,
    id: Option<u64>,
    #[serde(flatten)]
    content: ResponseContent,
}

impl Response {
    pub fn new(id: Option<u64>, success: impl Serialize) -> Response {
        Response {
            jsonrpc: Version::V2,
            id,
            content: ResponseContent::Success(serde_json::to_value(success).unwrap()),
        }
    }

    pub fn new_err(id: Option<u64>, error: ErrorObject) -> Response {
        Response {
            jsonrpc: Version::V2,
            id,
            content: ResponseContent::Error(error),
        }
    }
}

impl<'a> Into<HyperResponse<Body>> for &'a Response {
    fn into(self) -> HyperResponse<Body> {
        let body = Body::from(serde_json::to_vec(self).unwrap());
        HyperResponse::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }
}

#[derive(Debug, Serialize)]
pub enum ResponseContent {
    #[serde(rename = "result")]
    Success(Value),
    #[serde(rename = "error")]
    Error(ErrorObject),
}

#[derive(Debug, Serialize)]
pub struct ErrorObject {
    pub code: i64,
    pub message: Cow<'static, str>,
    pub data: Option<Value>,
}

impl ErrorObject {
    pub const PARSE_ERROR: ErrorObject = ErrorObject {
        code: -32700,
        message: Cow::Borrowed("Parse error"),
        data: None,
    };

    pub const INVALID_REQUEST: ErrorObject = ErrorObject {
        code: -32600,
        message: Cow::Borrowed("Invalid Request"),
        data: None,
    };

    pub const METHOD_NOT_FOUND: ErrorObject = ErrorObject {
        code: -32601,
        message: Cow::Borrowed("Method not found"),
        data: None,
    };

    pub const INVALID_PARAMS: ErrorObject = ErrorObject {
        code: -32602,
        message: Cow::Borrowed("Invalid params"),
        data: None,
    };

    pub const INTERNAL_ERROR: ErrorObject = ErrorObject {
        code: -32603,
        message: Cow::Borrowed("Internal error"),
        data: None,
    };

    pub fn custom<S>(code: i64, message: S, data: Option<impl Serialize>) -> ErrorObject
    where
        Cow<'static, str>: From<S>,
    {
        ErrorObject {
            code,
            message: message.into(),
            data: data.map(|s| serde_json::to_value(s).unwrap()),
        }
    }
}
