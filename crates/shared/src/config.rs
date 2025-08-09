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
    // 合约地址（可选为空，若为空从数据库 registry 读取或后续热更新）
    #[serde(default)] pub ticket_manager_addr: Option<String>,
    #[serde(default)] pub event_manager_addr: Option<String>,
    #[serde(default)] pub marketplace_addr: Option<String>,
    #[serde(default)] pub token_swap_addr: Option<String>,
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
