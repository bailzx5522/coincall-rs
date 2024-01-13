use lazy_static::lazy_static;
use std::env::var;

// sim trading
// WebSocket公共频道：wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999
// WebSocket私有频道：wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999
// WebSocket业务频道：wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999
// apikey = "84e5a285-8f21-4651-9e2c-e7920e3c78e4"
// secretkey = "0C29320B453826CE6318B8CA82BEE566"
// IP = ""
// 备注名 = "test"
// 权限 = "读取/交易"

// dotenv is a must run in every test otherwise the url will be mis-loaded
lazy_static! {
    pub static ref PUB_WS_URL: &'static str = {
        if var("OKEX_AWS").unwrap_or_else(|_| "0".to_string()) == "0" {
            // "wss://ws.okex.com:8443/ws/v5/public"
            "wss://ws.okx.com:8443/ws/v5/public"
        } else {
            // "wss://wsaws.okex.com:8443/ws/v5/public"
            "wss://wsaws.okx.com:8443/ws/v5/public"
        }
    };
    pub static ref PRIV_WS_URL: &'static str = {
        if var("OKEX_AWS").unwrap_or_else(|_| "0".to_string()) == "0" {
            // "wss://ws.okex.com:8443/ws/v5/private"
            "wss://ws.okx.com:8443/ws/v5/private"
        } else {
            // "wss://wsaws.okex.com:8443/ws/v5/private"
            "wss://wsaws.okx.com:8443/ws/v5/private"
        }
    };
    pub static ref REST_URL: &'static str = {
        if var("OKEX_AWS").unwrap_or_else(|_| "0".to_string()) == "0" {
            // "https://www.okex.com"
            "https://www.okx.com"
        } else {
            // "https://aws.okex.com"
            "https://aws.okx.com"
        }
    };
    pub static ref IS_AWS: bool = var("OKEX_AWS").unwrap_or_else(|_| "0".to_string()) != "0";
}
