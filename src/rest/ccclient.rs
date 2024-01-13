use super::{models::CCRequest};
use crate::credential::Credential;
use crate::error::OkExError;
use crate::urls::REST_URL;
use chrono::{SecondsFormat, Utc};
use derive_builder::Builder;
use fehler::{throw, throws};
use hyper::Method;
use log::error;
use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{from_str, to_string as to_jstring};
use serde_urlencoded::to_string as to_ustring;
use url::Url;

#[derive(Clone, Builder)]
pub struct CCExRest {
    client: Client,
    #[builder(default, setter(strip_option))]
    credential: Option<Credential>,
}

impl Default for CCExRest {
    fn default() -> Self {
        Self::new()
    }
}

impl CCExRest {
    pub fn new() -> Self {
        CCExRest {
            client: Client::new(),
            credential: None,
        }
    }

    pub fn with_credential(api_key: &str, api_secret: &str, passphrase: &str) -> Self {
        CCExRest {
            client: Client::new(),
            credential: Some(Credential::new(api_key, api_secret, passphrase)),
        }
    }

    pub fn builder() -> CCExRestBuilder {
        CCExRestBuilder::default()
    }

    #[throws(OkExError)]
    pub async fn request<R>(&self, req: R) -> R::Response
    where
        R: CCRequest,
        R::Response: DeserializeOwned,
    {
        let host = "https://api.coincall.com";
        let mut url = format!("{}{}", &*host, R::ENDPOINT);
        url.push_str(req.get_addition_uri().unwrap().as_str());
        let mut url = Url::parse(&url)?;
        if matches!(R::METHOD, Method::GET | Method::DELETE) && R::HAS_PAYLOAD {
            url.set_query(Some(&to_ustring(&req)?));
        }

        let body = match R::METHOD {
            Method::PUT | Method::POST => to_jstring(&req)?,
            _ => "".to_string(),
        };

        let mut builder = self.client.request(R::METHOD, url.clone());

        if R::SIGNED {
            let cred = self.get_credential()?;
            let timestamp = Utc::now().timestamp_millis();
            let (key, signature) = cred.cc_signature(R::METHOD, &timestamp, &url);

            builder = builder
                .header("X-CC-APIKEY", key)
                .header("sign", signature)
                .header("ts", timestamp)
                .header("X-REQ-TS-DIFF", 3000)
                .header("Content-Type", "application/json")
        }
        println!("request {:?}", builder);

        let resp = builder
            // .body(body)
            .send()
            .await?;
        // println!(" response {:?}, {}", resp, url.clone());
        self.handle_response(resp).await?
    }

    #[throws(OkExError)]
    fn get_credential(&self) -> &Credential {
        match self.credential.as_ref() {
            None => throw!(OkExError::NoApiKeySet),
            Some(c) => c,
        }
    }

    #[throws(OkExError)]
    async fn handle_response<T: DeserializeOwned>(&self, resp: Response) -> T {
        let payload = resp.text().await?;

        match from_str::<CCExResponseEnvolope<T>>(&payload) {
            Ok(v) => v.data,
            Err(e) => {
                error!("Cannot deserialize response from {}: {}", payload, e);
                throw!(OkExError::CannotDeserializeResponse(payload))
            }
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct CCExResponseEnvolope<T> {
    code: i32,
    msg: Option<String>,
    i18nArgs: Option<String>,
    data: T,
}
