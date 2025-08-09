use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: i64,
    pub organizer_wallet: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub venue: Option<String>,
    pub status: String,
}

#[derive(Debug)]
pub struct NewEvent<'a> {
    pub organizer_wallet: &'a str,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub venue: Option<&'a str>,
    pub status: &'a str,
}
