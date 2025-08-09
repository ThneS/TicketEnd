use axum::{routing::{get, post}, Router, extract::State, Json};
use shared::{AppConfig, db::pool::Db, repo::event_repo::EventRepo, domain::event::NewEvent};
use shared::seed;
use tracing_subscriber::{EnvFilter, fmt};
use std::sync::Arc;
use chrono::{Utc, Duration};

struct AppState { db: Db }

#[tokio::main]
async fn main() {
    init_tracing();
    let cfg = AppConfig::from_env();
    let db = Db::connect(&cfg).await.expect("db connect");
    db.migrate().await.expect("migrate");
    seed::seed(&db).await.expect("seed");
    let state = Arc::new(AppState { db });

    let app = Router::new()
        .route("/health", get(health))
        .route("/events/demo-create", post(demo_create_event))
        .with_state(state);

    let addr = cfg.listen_addr.parse().expect("listen addr");
    tracing::info!(?addr, "api listening");
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn health() -> &'static str { "ok" }

async fn demo_create_event(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let repo = EventRepo::new(&state.db);
    let now = Utc::now();
    let new = NewEvent { organizer_wallet: "0xDemo", start_time: now + Duration::hours(24), end_time: now + Duration::hours(30), venue: Some("Demo Venue"), status: "draft" };
    let ev = repo.insert(&new).await.expect("insert");
    Json(serde_json::json!({"id": ev.id, "status": ev.status}))
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).json().init();
}
