use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use crate::AppConfig;

#[derive(Clone)]
pub struct Db(pub Pool<Postgres>);

impl Db {
    pub async fn connect(cfg: &AppConfig) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&cfg.database_url)
            .await?;
        Ok(Self(pool))
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        // migrations 目录位于 shared crate 下，运行时当前工作目录在 workspace 根
        sqlx::migrate!("../../crates/shared/migrations").run(&self.0).await?;
        Ok(())
    }
}
