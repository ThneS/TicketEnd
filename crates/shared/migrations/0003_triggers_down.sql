-- 回滚 0003: 删除触发器与函数 (PostgreSQL)
DROP TRIGGER IF EXISTS trg_events_updated_at
ON events;
DROP TRIGGER IF EXISTS trg_tickets_updated_at
ON tickets;
DROP TRIGGER IF EXISTS trg_market_listings_updated_at
ON market_listings;
DROP TRIGGER IF EXISTS trg_trades_updated_at
ON trades;
DROP FUNCTION IF EXISTS set_updated_at
();
-- 注意：保留 updated_at 列，不做 DROP，以免破坏数据审计
