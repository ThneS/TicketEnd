use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub rpc_http_url: String,
    pub rpc_ws_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub qr_hmac_secret: String,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
}

fn default_listen_addr() -> String { "0.0.0.0:8080".into() }

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let fig = figment::Figment::new()
            .merge(figment::providers::Env::prefixed("OT_"));
        fig.extract().expect("config load failed")
    }
}
