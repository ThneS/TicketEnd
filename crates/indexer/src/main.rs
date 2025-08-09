use alloy::primitives::{Address, B256, U256};
use alloy::rpc::types::eth::Filter;
use alloy::sol_types::SolEvent;
use futures::TryStreamExt;
use shared::contracts::bindings::{EventManager, Marketplace, TicketManager, TokenSwap};
use shared::contracts::registry::resolve_addresses;
use shared::{contracts::provider::build_provider, db::pool::Db, AppConfig};
use sqlx::Acquire;
use tokio::time::{interval, sleep, Duration};
use tracing_subscriber::{fmt, EnvFilter};

// 简易游标表（若尚未建表，可后续迁移添加，这里先用临时表名占位）
// CREATE TABLE indexer_cursors(chain_id BIGINT PRIMARY KEY, last_block BIGINT NOT NULL);

const CONFIRM_DEPTH: i64 = 6; // 最小确认区块数，避免浅层重组

#[tokio::main]
async fn main() {
    init_tracing();
    let cfg = AppConfig::from_env();
    let db = Db::connect(&cfg).await.expect("db connect");
    db.migrate().await.expect("migrate");
    let provider = build_provider(&cfg).await.expect("provider");

    let chain_id = match provider.get_chain_id().await {
        Ok(id) => {
            tracing::info!(?id, "provider initialized");
            id
        }
        Err(e) => {
            tracing::error!(?e, "chain id error");
            return;
        }
    } as i64;

    let addrs = match resolve_addresses(&cfg, &db, chain_id).await {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(?e, "resolve addresses failed");
            return;
        }
    };
    tracing::info!(?addrs, "contract addresses loaded");

    tracing::info!("indexer init: ensure cursors");
    ensure_cursor(&db, chain_id).await.expect("ensure cursor");

    // 回填阶段
    if let Err(e) = backfill(&db, &provider, chain_id, &addrs).await {
        tracing::error!(?e, "backfill error");
    }

    // 增量轮询任务
    tracing::info!("start incremental loop");
    incremental_loop(db, provider, chain_id, addrs).await;
}

async fn ensure_cursor(db: &Db, chain_id: i64) -> anyhow::Result<()> {
    sqlx::query!("CREATE TABLE IF NOT EXISTS indexer_cursors(chain_id BIGINT PRIMARY KEY, last_block BIGINT NOT NULL)").execute(&db.0).await?;
    sqlx::query!(
        "INSERT INTO indexer_cursors(chain_id,last_block) VALUES($1,$2) ON CONFLICT DO NOTHING",
        chain_id,
        0_i64
    )
    .execute(&db.0)
    .await?;
    Ok(())
}

async fn load_cursor(db: &Db, chain_id: i64) -> anyhow::Result<i64> {
    let v = sqlx::query_scalar!(
        "SELECT last_block FROM indexer_cursors WHERE chain_id=$1",
        chain_id
    )
    .fetch_one(&db.0)
    .await?;
    Ok(v)
}

async fn save_cursor(db: &Db, chain_id: i64, block: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE indexer_cursors SET last_block=$2 WHERE chain_id=$1",
        chain_id,
        block
    )
    .execute(&db.0)
    .await?;
    Ok(())
}

struct Addresses {
    ticket: Address,
    event: Address,
    market: Address,
    swap: Address,
}

async fn backfill(
    db: &Db,
    provider: &shared::contracts::provider::SharedProvider,
    chain_id: i64,
    addrs: &shared::contracts::registry::ContractAddresses,
) -> anyhow::Result<()> {
    let start = load_cursor(db, chain_id).await?;
    let raw_latest = provider.get_block_number().await? as i64;
    if raw_latest <= CONFIRM_DEPTH {
        tracing::warn!(
            raw_latest,
            "chain height below confirm depth; skip backfill"
        );
        return Ok(());
    }
    let latest = raw_latest - CONFIRM_DEPTH; // 只处理已确认高度
    if start >= latest {
        tracing::info!("no backfill needed: start={start} latest={latest}");
        return Ok(());
    }
    tracing::info!(
        start,
        latest,
        head = raw_latest,
        depth = CONFIRM_DEPTH,
        "begin backfill range (finalized)"
    );

    let batch: i64 = 1_000;
    let mut from = start.max(0);
    while from < latest {
        let to = (from + batch).min(latest);
        fetch_and_process(db, provider, chain_id, addrs, from + 1, to).await?; // (from, to]
        save_cursor(db, chain_id, to).await?;
        from = to;
    }
    Ok(())
}

async fn incremental_loop(
    db: Db,
    provider: shared::contracts::provider::SharedProvider,
    chain_id: i64,
    addrs: shared::contracts::registry::ContractAddresses,
) {
    let mut intv = interval(Duration::from_secs(6));
    loop {
        intv.tick().await;
        match incremental_step(&db, &provider, chain_id, &addrs).await {
            Ok(()) => {}
            Err(e) => tracing::error!(?e, "incremental step error"),
        }
    }
}

async fn incremental_step(
    db: &Db,
    provider: &shared::contracts::provider::SharedProvider,
    chain_id: i64,
    addrs: &shared::contracts::registry::ContractAddresses,
) -> anyhow::Result<()> {
    let mut last = load_cursor(db, chain_id).await?;
    let raw_latest = provider.get_block_number().await? as i64;
    if raw_latest <= CONFIRM_DEPTH {
        return Ok(());
    }
    let latest = raw_latest - CONFIRM_DEPTH; // finalized tip
    if last >= latest {
        return Ok(());
    }
    let batch: i64 = 500;
    while last < latest {
        let to = (last + batch).min(latest);
        fetch_and_process(db, provider, chain_id, addrs, last + 1, to).await?;
        save_cursor(db, chain_id, to).await?;
        last = to;
    }
    Ok(())
}

// 收集所有关心事件的 topic0
fn topics(addrs: &shared::contracts::registry::ContractAddresses) -> Vec<Address> {
    vec![
        addrs.ticket_manager,
        addrs.event_manager,
        addrs.marketplace,
        addrs.token_swap,
    ]
}

async fn fetch_and_process(
    db: &Db,
    provider: &shared::contracts::provider::SharedProvider,
    chain_id: i64,
    addrs: &shared::contracts::registry::ContractAddresses,
    from: i64,
    to: i64,
) -> anyhow::Result<()> {
    use alloy::primitives::B256;
    let addresses = topics(addrs);
    // 构建过滤器（地址集合 + 区块范围）
    let filter = Filter::new()
        .address(addresses)
        .from_block(from)
        .to_block(to);
    let logs = provider.get_logs(&filter).await?;
    tracing::info!(count = logs.len(), from, to, "fetched logs");
    for lg in logs.into_iter() {
        if let Err(e) = process_log(db, chain_id, &lg).await {
            tracing::error!(?e, "process log error");
        }
    }
    Ok(())
}

async fn process_log(
    db: &Db,
    chain_id: i64,
    lg: &alloy::rpc::types::eth::Log,
) -> anyhow::Result<()> {
    use alloy::rpc::types::eth::Log;
    // 持久化原始记录 (避免重复: ON CONFLICT DO NOTHING)
    let tx_hash = format!("0x{:x}", lg.transaction_hash.unwrap_or_default());
    let primary_topic = lg
        .topics
        .get(0)
        .map(|t| format!("0x{:x}", t))
        .unwrap_or_default();
    sqlx::query!("INSERT INTO chain_logs(chain_id,block_number,tx_hash,log_index,primary_topic,contract_address,data) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT DO NOTHING",
        chain_id,
        lg.block_number.unwrap_or_default() as i64,
        tx_hash,
        lg.log_index.unwrap_or_default() as i32,
        primary_topic,
        format!("0x{:x}", lg.address),
        serde_json::json!({ "data": format!("0x{:x}", B256::from_slice(&lg.data)) })
    ).execute(&db.0).await?;

    // TODO: 根据 topic 匹配具体事件，解码后更新各业务表
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).json().init();
}
