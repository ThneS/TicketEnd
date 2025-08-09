use anyhow::Result;
use alloy::providers::{ProviderBuilder, Provider, WsConnect};
use alloy::transports::Transport;
use alloy::providers::fillers::{ChainIdFiller, GasFiller, BlobGasFiller, NonceFiller};
use alloy::providers::RootProvider;
use alloy::transports::http::Http;
use crate::AppConfig;
use std::sync::Arc;

pub type SharedProvider = Arc<RootProvider<Box<dyn Transport>>>;

pub async fn build_provider(cfg: &AppConfig) -> Result<SharedProvider> {
    // HTTP 主 provider（可扩展为多层回退）
    let http = Http::new(cfg.rpc_http_url.parse()?);
    let ws = WsConnect::new(cfg.rpc_ws_url.as_str());

    // RootProvider 可使用 http + ws 组合；这里简化为 http 主 + ws 事件
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(http);
    let provider: RootProvider<_> = provider;

    Ok(Arc::new(provider.boxed()))
}
