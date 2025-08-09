use crate::{db::pool::Db, repo::event_repo::EventRepo, domain::event::NewEvent};
use anyhow::Result;
use chrono::{Utc, Duration};

pub async fn seed(db: &Db) -> Result<()> {
    let repo = EventRepo::new(db);
    let now = Utc::now();
    // 简单幂等：检查是否已有任意活动
    let existing = sqlx::query_scalar!("SELECT id FROM events LIMIT 1").fetch_optional(&db.0).await?;
    if existing.is_some() { return Ok(()); }

    let new = NewEvent { organizer_wallet: "0xSeed", start_time: now + Duration::hours(48), end_time: now + Duration::hours(52), venue: Some("Seed Venue"), status: "draft" };
    let _ = repo.insert(&new).await?;
    Ok(())
}
