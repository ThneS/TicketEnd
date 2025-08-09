use crate::db::pool::Db;
use crate::AppConfig;
use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use sqlx::Row;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

#[derive(Clone, Debug)]
pub struct ContractAddresses {
    pub ticket_manager: Address,
    pub event_manager: Address,
    pub marketplace: Address,
    pub token_swap: Address,
}

impl ContractAddresses {
    pub fn from_config(cfg: &AppConfig) -> Option<Self> {
        let parse = |opt: &Option<String>| -> Option<Address> { opt.as_ref()?.parse().ok() };
        Some(Self {
            ticket_manager: parse(&cfg.ticket_manager_addr)?,
            event_manager: parse(&cfg.event_manager_addr)?,
            marketplace: parse(&cfg.marketplace_addr)?,
            token_swap: parse(&cfg.token_swap_addr)?,
        })
    }
}

pub async fn load_from_db(db: &Db, chain_id: i64) -> Result<ContractAddresses> {
    async fn get(db: &Db, chain_id: i64, name: &str) -> Result<Address> {
        let rec =
            sqlx::query("SELECT address FROM contract_registry WHERE chain_id=$1 AND name=$2")
                .bind(chain_id)
                .bind(name)
                .fetch_one(&db.0)
                .await?;
        let addr: String = rec.get("address");
        addr.parse()
            .map_err(|_| anyhow!("invalid address in registry: {}", name))
    }
    Ok(ContractAddresses {
        ticket_manager: get(db, chain_id, "TicketManager").await?,
        event_manager: get(db, chain_id, "EventManager").await?,
        marketplace: get(db, chain_id, "Marketplace").await?,
        token_swap: get(db, chain_id, "TokenSwap").await?,
    })
}

pub async fn resolve_addresses(
    cfg: &AppConfig,
    db: &Db,
    chain_id: i64,
) -> Result<ContractAddresses> {
    if let Some(a) = ContractAddresses::from_config(cfg) {
        return Ok(a);
    }
    load_from_db(db, chain_id).await
}

// -------- 热更新缓存 --------
#[derive(Clone)]
pub struct AddressCache(Arc<RwLock<Option<ContractAddresses>>>);

impl AddressCache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }
    pub fn get(&self) -> Option<ContractAddresses> {
        self.0.read().ok().and_then(|g| g.clone())
    }
    pub fn set(&self, v: ContractAddresses) {
        if let Ok(mut g) = self.0.write() {
            *g = Some(v);
        }
    }
}

pub async fn spawn_registry_watcher(
    cache: AddressCache,
    cfg: AppConfig,
    db: Db,
    chain_id: i64,
    redis_url: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // 初始化加载
        match resolve_addresses(&cfg, &db, chain_id).await {
            Ok(a) => {
                cache.set(a);
            }
            Err(e) => tracing::error!(?e, "initial address load failed"),
        }
        let client = match redis::Client::open(redis_url) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(?e, "redis client fail");
                return;
            }
        };
        let mut conn = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(?e, "redis conn fail");
                return;
            }
        };
        let mut pubsub = conn.as_pubsub();
        if let Err(e) = pubsub.subscribe("contract_registry_update").await {
            tracing::error!(?e, "subscribe fail");
            return;
        }
        tracing::info!("registry watcher started");
        loop {
            match pubsub.on_message().next().await {
                Some(msg) => {
                    let payload: String = msg.get_payload().unwrap_or_default();
                    tracing::info!(payload, "contract_registry_update received");
                    match resolve_addresses(&cfg, &db, chain_id).await {
                        Ok(a) => cache.set(a),
                        Err(e) => tracing::error!(?e, "reload addresses failed"),
                    }
                }
                None => {
                    tracing::warn!("pubsub stream ended");
                    break;
                }
            }
        }
    })
}
