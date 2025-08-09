// 简易运维脚本：更新某个链上合约地址并广播 Redis 通知
// 运行方式：cargo run --bin update_contract_registry -- <chain_id> <Name> <address>
// 可在后续改造成独立 crate/bin，这里先提供逻辑示例。

use anyhow::{Context, Result};
use std::env;
use sqlx::{Pool, Postgres};
use dotenvy::dotenv;
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 { eprintln!("usage: update_contract_registry <chain_id> <Name> <address>"); std::process::exit(1); }
    let chain_id: i64 = args[1].parse()?;
    let name = &args[2];
    let address = &args[3];

    let database_url = env::var("OT_DATABASE_URL").context("OT_DATABASE_URL not set")?;
    let redis_url = env::var("OT_REDIS_URL").context("OT_REDIS_URL not set")?;

    let pool = Pool::<Postgres>::connect(&database_url).await?;
    sqlx::query!("INSERT INTO contract_registry(chain_id,name,address,updated_at) VALUES ($1,$2,$3,NOW()) ON CONFLICT (chain_id,name) DO UPDATE SET address=EXCLUDED.address, updated_at=NOW()", chain_id, name, address).execute(&pool).await?;
    println!("updated registry: {chain_id} {name} {address}");

    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_async_connection().await?;
    let _: () = conn.publish("contract_registry_update", format!("{chain_id}:{name}")).await?;
    println!("published contract_registry_update");
    Ok(())
}
