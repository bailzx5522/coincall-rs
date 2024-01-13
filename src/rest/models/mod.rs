mod account;
mod market;
mod public;
mod trade;
mod ccpublic;
mod ccprivate;

pub use account::*;
pub use market::*;
pub use public::*;
pub use trade::*;
pub use ccpublic::*;
pub use ccprivate::*;

use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Request: Serialize {
    const METHOD: Method;
    const SIGNED: bool = false;
    const ENDPOINT: &'static str;
    const HAS_PAYLOAD: bool = true;
    type Response: DeserializeOwned;

    #[inline]
    fn no_payload(&self) -> bool {
        !Self::HAS_PAYLOAD
    }
}

pub trait CCRequest: Serialize {
    const METHOD: Method;
    const SIGNED: bool = false;
    const ENDPOINT: &'static str;
    const HAS_PAYLOAD: bool = true;
    type Response: DeserializeOwned;

    #[inline]
    fn no_payload(&self) -> bool {
        !Self::HAS_PAYLOAD
    }
    fn get_addition_uri(&self)->Option<String>{
        Some(String::new())
    }
    
}