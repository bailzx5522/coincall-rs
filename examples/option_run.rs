use std::{sync::{Arc, Mutex}, collections::HashMap};

use anyhow::Error;
use chrono::format::Item;
use fehler::throws;
use futures::{SinkExt, StreamExt, future::join};
use http::response;
use hyper::client;
use okex::{websocket::{models::{Ticker, Order}, Channel, Command, Message, OkExWebsocket, CCChannel, CCMessage, OrderBookData}, OkExError, enums::InstType, rest::{CCExRest, TestRequest, AccountInfoRequest, InstrumentsRequest, CCInstrumentsRequest}};
use okex::{websocket::{CCExWebsocket}};
use okex::websocket::CCCommand;
use serde_json::from_value;
use tokio::sync::mpsc;


// symbol -> Orderbookdata
// ex: btc-usd-240112-45000-c -> asks,bids
type ShardedDb = Arc<Vec<Mutex<HashMap<String, OrderBookData>>>>;

#[throws(Error)]
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let ak = env::var("CC_KEY").unwrap();
    let sk = env::var("CC_SECRET").unwrap();
    println!("key&sec {}:{}", ak, sk);
    return;
    // rest api for test
    //{
    //    let client  = CCExRest::with_credential(ak, sk, "");
    //    // let resp = client.request(TestRequest::time()).await?;

    //    // let resp = client.request(AccountInfoRequest::new()).await?;
    //    let resp = client.request(CCInstrumentsRequest::get_inst("SOL")).await?;
    //    resp.iter().all(|x| {
    //        println!("{}" , x.symbol_name);
    //        return true;
    //    }
    //    );
    //    // println!("--------------------{:?}", resp.);
    //}

    let (tx_cc, mut rx) = mpsc::channel(1024);
    let tx_ok = tx_cc.clone();
    let orderbook_db: ShardedDb = Arc::new(Mutex::new(HashMap::new()));

    let ccex = tokio::spawn(async move {
        let mut client = CCExWebsocket::with_credential(ak, sk).await.unwrap();
        // client
        //     .send(CCCommand::HeartBeat)
        //     .await.unwrap();
        client
            .send(CCCommand::subscribe(CCChannel::orderbook("SOLUSD-12JAN24-81.0-C")))
            .await.unwrap();
         while let Some(x) = client.next().await {
            match x {
                Ok(m) => {
                    match m {
                        CCMessage::Data { dt, c, d } => {
                            let x: OrderBookData = from_value(d).unwrap();
                            tx_cc.send(x).await;
                        }
                        CCMessage::Pong { c, rc } => {}
                        _ => {}
                    }
                }
                Err(e) => {println!("============={:?}", e)}
            }
        }
    });

    // okx ws
    let ok_handler = tokio::spawn(async move {
        let mut ok_client = OkExWebsocket::new().await.unwrap();

        ok_client
            .send(Command::subscribe(vec![Channel::Tickers {
                inst_type: Some(InstType::Option),
                inst_id: "BTC-USD-240112-45000-C".to_string(),
            }]))
            .await.unwrap();

        while let Some(x) = ok_client.next().await {
            match x {
                Ok(m) => {
                    match m {
                        Message::Data { arg, mut data, .. } => {
                            assert!(matches!(arg, Channel::Tickers { .. }));
                            let data = data.pop().unwrap();
                            let x: Ticker = from_value(data).unwrap();
                            let y = x.ticker_to_orderbook();
                            // println!("--------- okx ticker{:?}", x);
                            tx_ok.send(y).await;
                        }
                        Message::Error { code, msg, .. } => {
                            println!("Error {}: {}", code, msg)
                        }
                        Message::Event { .. } => {}
                        _ => unreachable!(),
                    }
                }
                Err(e) => {
                    println!("okex connection return error {:?}", e)

                }
            }
        }
    });

    // let balance_handler = tokio::spawn(async move {
    // });
    // let position_handler = tokio::spawn(async move {
    // });
    // // order events
    // let order_handler = tokio::spawn(async move {
    // });

    let consumer_handler = tokio::spawn(async move {
        while let Some(orderbook) = rx.recv().await {
            println!("process orderbook quota {:?}", orderbook);
            // process cc/okx orderbook
            // get balance between exchagnes
            // place order decisions
            // update local orders
        }
    });


    // ccex.await.unwrap();
    ok_handler.await.unwrap();

}
