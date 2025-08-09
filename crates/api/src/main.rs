use alloy::primitives::U256;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use shared::contracts::provider::build_provider;
use shared::contracts::registry::{resolve_addresses, spawn_registry_watcher, AddressCache};
use shared::seed;
use shared::{db::pool::Db, domain::event::NewEvent, repo::event_repo::EventRepo, AppConfig};
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter}; // 示例保留

struct AppState {
    db: Db,
    cache: AddressCache,
    cfg: AppConfig,
    chain_id: i64,
}

#[tokio::main]
async fn main() {
    init_tracing();
    let cfg = AppConfig::from_env();
    let db = Db::connect(&cfg).await.expect("db connect");
    db.migrate().await.expect("migrate");
    seed::seed(&db).await.expect("seed");

    // 构建 provider 获取真实链 ID
    let provider = build_provider(&cfg).await.expect("provider");
    let chain_id = match provider.get_chain_id().await {
        Ok(id) => {
            tracing::info!(?id, "provider initialized");
            id as i64
        }
        Err(e) => {
            tracing::error!(?e, "chain id error");
            return;
        }
    };

    let cache = AddressCache::new();
    if let Ok(a) = resolve_addresses(&cfg, &db, chain_id).await {
        cache.set(a);
    }

    // 启动 Redis PubSub 热更新 watcher（后台任务）
    let _watcher = spawn_registry_watcher(
        cache.clone(),
        cfg.clone(),
        db.clone(),
        chain_id,
        cfg.redis_url.clone(),
    )
    .await;

    let state = Arc::new(AppState {
        db,
        cache,
        cfg: cfg.clone(),
        chain_id,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/events/demo-create", post(demo_create_event))
        .route("/contracts/addresses", get(get_addresses))
        .with_state(state);

    let addr = cfg.listen_addr.parse().expect("listen addr");
    tracing::info!(?addr, chain_id, "api listening");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn demo_create_event(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let repo = EventRepo::new(&state.db);
    let now = Utc::now();
    let new = NewEvent {
        organizer_wallet: "0xDemo",
        start_time: now + Duration::hours(24),
        end_time: now + Duration::hours(30),
        venue: Some("Demo Venue"),
        status: "draft",
    };
    let ev = repo.insert(&new).await.expect("insert");
    Json(serde_json::json!({"id": ev.id, "status": ev.status}))
}

async fn get_addresses(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    if let Some(a) = state.cache.get() {
        Json(serde_json::json!({
            "chain_id": state.chain_id,
            "ticket_manager": format!("0x{:x}", a.ticket_manager),
            "event_manager": format!("0x{:x}", a.event_manager),
            "marketplace": format!("0x{:x}", a.marketplace),
            "token_swap": format!("0x{:x}", a.token_swap),
        }))
    } else {
        Json(serde_json::json!({"error": "addresses_not_loaded", "chain_id": state.chain_id}))
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).json().init();
}
