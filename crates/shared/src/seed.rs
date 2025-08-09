use crate::{db::pool::Db, domain::event::NewEvent, repo::event_repo::EventRepo};
use anyhow::Result;
use chrono::{Duration, Utc};

pub async fn seed(db: &Db) -> Result<()> {
    seed_events(db).await?;
    seed_contract_registry(db).await?;
    Ok(())
}

async fn seed_events(db: &Db) -> Result<()> {
    let repo = EventRepo::new(db);
    let now = Utc::now();
    // 简单幂等：检查是否已有任意活动
    let existing = sqlx::query_scalar!("SELECT id FROM events LIMIT 1")
        .fetch_optional(&db.0)
        .await?;
    if existing.is_some() {
        return Ok(());
    }

    let new = NewEvent {
        organizer_wallet: "0xSeed",
        start_time: now + Duration::hours(48),
        end_time: now + Duration::hours(52),
        venue: Some("Seed Venue"),
        status: "draft",
    };
    let _ = repo.insert(&new).await?;
    Ok(())
}

async fn seed_contract_registry(db: &Db) -> Result<()> {
    // 只插入一次：如果表为空则写入示例记录（使用 0x000... 占位，后续需运维替换）
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM contract_registry")
        .fetch_one(&db.0)
        .await?;
    if count > 0 {
        return Ok(());
    }
    // 使用 0 作为占位 chain_id（真实链上启动后将通过实际 chain_id + 运维脚本覆盖）
    let chain_id: i64 = 0;
    let placeholder = "0x0000000000000000000000000000000000000000";
    let names = ["TicketManager", "EventManager", "Marketplace", "TokenSwap"];
    for name in names {
        sqlx::query!(
            "INSERT INTO contract_registry(chain_id, name, address) VALUES ($1,$2,$3)",
            chain_id,
            name,
            placeholder
        )
        .execute(&db.0)
        .await?;
    }
    Ok(())
}
