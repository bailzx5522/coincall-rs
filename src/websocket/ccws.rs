// mod message;

use super::message::{Action, Message};
use std::pin::Pin;
use chrono::Utc;
use log::trace;
use serde::{Serialize, Deserialize};
use serde_json::{to_string, from_str};
use tokio_tungstenite::connect_async;
use url::Url;
use fehler::{throw, throws};
use futures::sink::Sink;
use futures::stream::Stream;
use anyhow::Error;
use futures::task::{Context, Poll};
use tungstenite::protocol::Message as WSMessage;

use crate::{credential::Credential, error::CCExError};

use super::{WSStream, Channel};

pub struct CCExWebsocket{
    credential: Option<Credential>,
    inner: WSStream,
}

impl CCExWebsocket {
    #[throws(CCExError)]
    pub async fn new() -> Self {
        Self::new_impl("", "").await?
    } 

    #[throws(CCExError)]
    pub async fn with_credential(api_key: &str, api_secret: &str) -> Self {
        let c = Self::new_impl(api_key, api_secret).await?;
        // c.credential = Some(Credential::new(api_key, api_secret, ""));
        c
    }

    #[throws(CCExError)]
    async fn new_impl(api_key: &str, api_secret: &str) -> Self {
        let c = Credential::new(api_key, api_secret, "");
        let ts = Utc::now().timestamp_millis();
        let (_, sign) = c.cc_ws_signature(&ts);
        let url = format!("wss://ws.coincall.com/options?code=10&uuid={}&ts={}&sign={}&apiKey={}", api_key, ts, sign, api_key);
        let (stream, resp) = connect_async(Url::parse(&url).unwrap()).await?;
        println!("--------------- ws resp {:?} {}", resp, url);
        Self {
            credential: Some(c),
            inner: stream,
        }
    }
    #[throws(CCExError)]
    fn get_credential(&self) -> &Credential {
        match self.credential.as_ref() {
            None => throw!(CCExError::NoApiKeySet),
            Some(c) => c,
        }
    }
}

impl Sink<CCCommand> for CCExWebsocket {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        println!("--------------- ready");
        let inner = Pin::new(&mut self.inner);
        inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn start_send(mut self: Pin<&mut Self>, item: CCCommand) -> Result<(), Self::Error> {
        let command = match &item {
            &CCCommand::HeartBeat => "{'action':'heartbeat'}".to_string(),
            command => to_string(command)?,
        };
        println!("--------------- start_send {}", command);
        trace!("Sending '{}' through websocket", command);
        let inner = Pin::new(&mut self.inner);
        Ok(inner.start_send(WSMessage::Text(command))?)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        println!("--------------- poll_flush");
        let inner = Pin::new(&mut self.inner);
        inner.poll_flush(cx).map_err(|e| e.into())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        println!("--------------- poll_close");
        let inner = Pin::new(&mut self.inner);
        inner.poll_close(cx).map_err(|e| e.into())
    }
}

impl Stream for CCExWebsocket {
    type Item = Result<CCMessage, CCExError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let inner = Pin::new(&mut self.inner);
        let poll = inner.poll_next(cx);
        match poll {
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(Some(Ok(m))) => match parse_message(m) {
                Ok(m) => Poll::Ready(Some(Ok(m))),
                Err(e) => Poll::Ready(Some(Err(e))),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

use serde_json::Value;
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorLiteral {
    Error,
}

#[derive(Clone, Debug, Deserialize)]
pub struct  LevelInfo{
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub pr: f64,
    #[serde(deserialize_with = "crate::parser::from_str")]
    pub sz: f64
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct  OrderBookData {
    pub ts: i64,
    pub s: String,
    pub asks: Vec<LevelInfo>,
    pub bids: Vec<LevelInfo>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginLiteral {
    Login,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum CCMessage<Row = Value> {
    Event {
        event: String,
        arg: Channel,
    },
    Login {
        event: LoginLiteral,
        code: String,
        msg: String,
    },
    Error {
        event: ErrorLiteral,
        code: String,
        msg: String,
    },
    Data {
        dt: i32,
        c: i32,
        d: Row,
    },
    Result {
        action: String,
        #[serde(rename = "dataType")]
        data_type: Option<String>,
        payload: Row,
        result: String
    },
    Pong {
        c : i32,
        rc: i32
    },
}

#[throws(CCExError)]
fn parse_message(msg: WSMessage) -> CCMessage {
    match msg {
        WSMessage::Text(message) => match message.as_str() {
            // "pong" => CCMessage::Pong,
            others => match from_str(others) {
                Ok(r) => r,
                Err(_) => unreachable!("Cannot deserialize message from OkEx: '{}'", others),
            },
        },
        WSMessage::Close(_) => {println!("----------------error");throw!(CCExError::WebsocketClosed)},
        WSMessage::Binary(_) => {println!("----------------error");throw!(CCExError::UnexpectedWebsocketBinaryMessage)},
        WSMessage::Ping(_) => throw!(CCExError::UnexpectedWebsocketPingMessage),
        WSMessage::Pong(_) => throw!(CCExError::UnexpectedWebsocketPongMessage),
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
pub enum CCCommand {
    // Subscribe(CCChannel),
    Subscribe { 
        #[serde(rename = "dataType")]
        data_type : String,
        payload : Payload},
    Login(Vec<LoginArgs>),
    HeartBeat,
}

impl CCCommand {
    pub fn subscribe(topics: CCChannel) -> CCCommand {
        CCCommand::Subscribe{data_type : topics.data_type, payload : topics.payload}
    }

    #[throws(CCExError)]
    pub fn login(client: &CCExWebsocket) -> CCCommand {
        let cred = client.get_credential()?;
        let ts = Utc::now().timestamp_millis();
        let (key, sign) = cred.cc_ws_signature(&ts);
        let url = format!("wss://ws.coincall.com/options?code=10&uuid={}&ts={}&sign={}&apiKey={}", key, ts, sign, key);
        print!("---------------{}", url);

        Self::Login(vec![LoginArgs {
            api_key: key.into(),
            passphrase: cred.passphrase().into(),
            timestamp: ts.to_string(),
            sign: sign,
        }])
    }

    pub fn heartbeat() -> CCCommand {
        CCCommand::HeartBeat
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginArgs {
    api_key: String,
    passphrase: String,
    timestamp: String,
    sign: String,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Payload{
    #[serde(rename = "symbol")]
    Symbol(String)
}
pub struct CCChannel {
    // { "action":"subscribe", "dataType":"orderBook", "payload":{ "symbol":"BTCUSD-27MAY23-26000-C" } }
    // {"action":"subscribe","data_type":"orderBook","payload":{"symbol":"BTCUSD-5Jan24-45000-C"}}
    data_type: String,
    payload: Payload,

    // #[serde(rename = "books50-l2-tbt")]
    // Books50L2Tbt {
    //     #[serde(rename = "instId")]
    //     inst_id: String,
    // },
    // #[serde(rename = "books-l2-tbt")]
    // BooksL2Tbt {
    //     #[serde(rename = "instId")]
    //     inst_id: String,
    // },
    // #[serde(rename = "instruments")]
    // Instruments {
    //     #[serde(rename = "instType")]
    //     inst_type: InstType,
    // },
    // #[serde(rename = "orders")]
    // Orders {
    //     #[serde(rename = "instType")]
    //     inst_type: InstType,
    //     uly: Option<String>,
    //     #[serde(rename = "instId")]
    //     inst_id: Option<String>,
    // },
    // #[serde(rename = "price-limit")]
    // PriceLimit {
    //     #[serde(rename = "instId")]
    //     inst_id: String,
    // },
    // #[serde(rename = "tickers")]
    // Tickers {
    //     #[serde(rename = "instType")]
    //     inst_type: InstType,
    //     #[serde(rename = "instId")]
    //     inst_id: String,
    // },
}

impl CCChannel {

    pub fn orderbook(symbol: &str) -> Self {
        Self {
            data_type: "orderBook".to_string(),
            payload: Payload::Symbol(symbol.into()),
        }
    }

    // pub fn books50_l2_tbt(inst_id: &str) -> Self {
    //     Self::Books50L2Tbt {
    //         inst_id: inst_id.into(),
    //     }
    // }

    // pub fn books_l2_tbt(inst_id: &str) -> Self {
    //     Self::BooksL2Tbt {
    //         inst_id: inst_id.into(),
    //     }
    // }

    // pub fn instruments(inst_type: InstType) -> Self {
    //     Self::Instruments { inst_type }
    // }

    // pub fn tickers(inst_type: InstType, inst_id: &str) -> Self {
    //     Self::Tickers {
    //         inst_type,
    //         inst_id: inst_id.into(),
    //     }
    // }

    // pub fn price_limit(inst_id: &str) -> Self {
    //     Self::PriceLimit {
    //         inst_id: inst_id.into(),
    //     }
    // }
}
