use axum::{routing::post, Router};
use tracing_subscriber::{fmt, EnvFilter};
use shared::{AppConfig, db::pool::Db};
use std::sync::Arc;

struct VerifierState { db: Db }

#[tokio::main]
async fn main() {
    init_tracing();
    let cfg = AppConfig::from_env();
    let db = Db::connect(&cfg).await.expect("db connect");
    db.migrate().await.expect("migrate");
    let state = Arc::new(VerifierState { db });

    let app = Router::new()
        .route("/verify/scan", post(scan_stub))
        .with_state(state);
    let addr = cfg.listen_addr.parse().expect("listen addr");
    tracing::info!(?addr, "verifier listening");
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn scan_stub() -> &'static str { "stub" }

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).json().init();
}
