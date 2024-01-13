use super::super::Request;
use crate::enums::{Alias, CtType, InstType, InstrumentState, OptType};
use http::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRequest {
}

impl TestRequest {
    pub fn time() -> Self{
        Self {}
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeResponse{
    // #[serde(deserialize_with = "crate::parser::from_str_opt")]
    pub server_time: i64,
}

impl Request for TestRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = false;
    const ENDPOINT: &'static str = "/time";
    const HAS_PAYLOAD: bool = false;
    type Response = TimeResponse;
}
