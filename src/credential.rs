use base64::encode;
use futures::io::AsyncReadExt;
use http::Method;
use ring::hmac;
use url::Url;
use hex;

#[derive(Clone, Debug)]
pub struct Credential {
    key: String,
    secret: String,
    passphrase: String,
}

impl Credential {
    pub(crate) fn new(key: &str, secret: &str, password: &str) -> Self {
        Self {
            key: key.into(),
            secret: secret.into(),
            passphrase: password.into(),
        }
    }

    pub(crate) fn passphrase(&self) -> &str {
        &self.passphrase
    }

    pub(crate) fn signature(
        &self,
        method: Method,
        timestamp: &str,
        url: &Url,
        body: &str,
    ) -> (&str, String) {
        // sign=CryptoJS.enc.Base64.stringify(CryptoJS.HmacSHA256(timestamp + 'GET' + '/users/self/verify' + body, SecretKey))
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret.as_bytes());
        let sign_message = match url.query() {
            Some(query) => format!(
                "{}{}{}?{}{}",
                timestamp,
                method.as_str(),
                url.path(),
                query,
                body
            ),
            None => format!("{}{}{}{}", timestamp, method.as_str(), url.path(), body),
        };

        let signature = encode(hmac::sign(&signed_key, sign_message.as_bytes()).as_ref());
        (self.key.as_str(), signature)
    }
    
    pub(crate) fn cc_signature(
        &self,
        method: Method,
        timestamp: &i64,
        url: &Url,
    ) -> (&str, String) {
        // msg = POST/open/futures/leverage/set/v1?leverage=1&symbol=BTCUSD&uuid=xdtHWn32rsuDQConutzl9JDZB+Y1leitFl356YHrmts=&ts=1688436087184&x-req-ts-diff=3000
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret.as_bytes());
        let sign_message = match url.query() {
            Some(query) => format!(
                "{}{}?{}&uuid={}&ts={}&x-req-ts-diff={}",
                method.as_str(),
                url.path(),
                query,
                self.key.as_str(),
                timestamp,
                3000,
            ),
            None => format!(
                "{}{}?uuid={}&ts={}&x-req-ts-diff={}",
                method.as_str(),
                url.path(),
                self.key.as_str(),
                timestamp,
                3000,
            ),
        };
        // let sign_message = "GET/open/account/summary/v1?uuid=YKFRiUGcLWfIprz7PfEA+9ExFGQSZzWP8qB5X++0+M8=&ts=1703952039202&x-req-ts-diff=3000";
        // let signature = encode(hmac::sign(&signed_key, sign_message.as_bytes()).as_ref());
        let signature = hmac::sign(&signed_key, sign_message.as_bytes());
        let  s = hex::encode_upper(signature.as_ref());
        let signature = s.to_uppercase();
        (self.key.as_str(), signature)
    }

    pub fn cc_ws_signature(&self, ts: &i64) -> (&str, String) {
        let sign_message = format!("GET/users/self/verify?uuid={}&ts={}", self.key.as_str(), ts);
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret.as_bytes());
        let signature = hmac::sign(&signed_key, sign_message.as_bytes());
        let  s = hex::encode_upper(signature.as_ref());
        let signature = s.to_uppercase();
        (self.key.as_str(), signature)
    }
}
