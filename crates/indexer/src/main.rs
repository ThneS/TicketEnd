use tracing_subscriber::{fmt, EnvFilter};
use shared::{AppConfig, db::pool::Db};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    init_tracing();
    let cfg = AppConfig::from_env();
    let db = Db::connect(&cfg).await.expect("db connect");
    db.migrate().await.expect("migrate");
    tracing::info!("indexer start (stub)");
    // TODO: 初始化 provider, 回填与订阅流程
    loop {
        // 占位：未来放置抓取区块逻辑
        sleep(Duration::from_secs(10)).await;
        tracing::debug!("indexer heartbeat");
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).json().init();
}
