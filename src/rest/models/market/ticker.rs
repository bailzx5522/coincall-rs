use super::super::Request;
use crate::websocket::LevelInfo;
use crate::{enums::InstType, websocket::OrderBookData};
use crate::parser::ts_milliseconds;
use chrono::{DateTime, Utc};
use http::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerRequest {
    pub inst_id: String,
}

impl TickerRequest {
    pub fn inst_id(inst_id: &str) -> Self {
        Self {
            inst_id: inst_id.into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TickerResponse {
    pub inst_type: InstType,
    pub inst_id: String,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub last: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub last_sz: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub ask_px: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub ask_sz: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub bid_px: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub bid_sz: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub open24h: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub high24h: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub low24h: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub vol_ccy24h: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub vol24h: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub sod_utc0: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub sod_utc8: f64,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub ts: DateTime<Utc>,
}

impl  TickerResponse {
    
    pub fn ticker_to_orderbook(self: &Self) -> OrderBookData {
        return OrderBookData { ts: self.ts.timestamp_micros(), s: self.inst_id.to_string(), asks: vec![LevelInfo{pr:self.ask_px, sz:self.ask_sz}], bids: vec![LevelInfo{pr: self.bid_px, sz:self.bid_sz}] };
    }
}

impl Request for TickerRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = false;
    const ENDPOINT: &'static str = "/api/v5/market/ticker";
    const HAS_PAYLOAD: bool = true;
    type Response = [TickerResponse; 1];
}
