-- 回滚 0002: 删除创建的唯一索引 (PostgreSQL)
DROP INDEX IF EXISTS uniq_market_listing_active_ticket;
DROP INDEX IF EXISTS uniq_verification_ticket_nonce;
DROP INDEX IF EXISTS uniq_events_organizer_chain_id;
DROP INDEX IF EXISTS uniq_trades_tx_listing;
