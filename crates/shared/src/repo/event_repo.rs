use crate::db::pool::Db;
use crate::domain::event::{Event, NewEvent};
use anyhow::Result;
use sqlx::Row;

pub struct EventRepo<'a> { pub db: &'a Db }

impl<'a> EventRepo<'a> {
    pub fn new(db: &'a Db) -> Self { Self { db } }

    pub async fn insert(&self, new: &NewEvent<'_>) -> Result<Event> {
        let rec = sqlx::query!(r#"INSERT INTO events (organizer_wallet, start_time, end_time, venue, status)
            VALUES ($1,$2,$3,$4,$5) RETURNING id, organizer_wallet, start_time, end_time, venue, status"#,
            new.organizer_wallet, new.start_time, new.end_time, new.venue, new.status)
            .fetch_one(&self.db.0).await?;
        Ok(Event { id: rec.id, organizer_wallet: rec.organizer_wallet, start_time: rec.start_time, end_time: rec.end_time, venue: rec.venue, status: rec.status })
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Event>> {
        let rec = sqlx::query!("SELECT id, organizer_wallet, start_time, end_time, venue, status FROM events WHERE id=$1", id)
            .fetch_optional(&self.db.0).await?;
        Ok(rec.map(|r| Event { id: r.id, organizer_wallet: r.organizer_wallet, start_time: r.start_time, end_time: r.end_time, venue: r.venue, status: r.status }))
    }
}
