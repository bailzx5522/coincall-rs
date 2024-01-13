use crate::rest::CCRequest;

use super::super::Request;
use http::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountInfoRequest {
    // pub ccy: Option<String>,
}

impl AccountInfoRequest {
    pub fn new() -> Self
    {
        Self{}
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfoResponse {
    pub name: String,
    pub email: String,
    pub user_id: i64,
}

impl Request for AccountInfoRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/open/user/info/v1";
    const HAS_PAYLOAD: bool = false;
    type Response = AccountInfoResponse;
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CCInstrumentsRequest {
    pub inst: String,
}

impl CCInstrumentsRequest {
    pub fn get_inst(inst: &str) -> Self
    {
        Self{
            inst: inst.into()
        }
    }

}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CCInstrumentsResponse {
    pub base_currency: String,
    // #[serde(deserialize_with = "crate::parser::from_str")]
    pub expiration_timestamp: i64,
    // #[serde(deserialize_with = "crate::parser::from_str")]
    pub strike: f64,
    pub symbol_name: String,
    // #[serde(deserialize_with = "crate::parser::from_str")]
    pub is_active:bool,
    // #[serde(deserialize_with = "crate::parser::from_str")]
    pub min_qty:f64,
    // #[serde(deserialize_with = "crate::parser::from_str")]
    pub tick_size:f64,
    
}

impl CCRequest for CCInstrumentsRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/open/option/getInstruments/";
    const HAS_PAYLOAD: bool = false;
    type Response = Vec<CCInstrumentsResponse>;

    fn get_addition_uri(&self)-> Option<String> {
        return Some(self.inst.clone());
    }
}
