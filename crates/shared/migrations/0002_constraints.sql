-- 0002_constraints (PostgreSQL ONLY)
-- 业务唯一性与一致性索引
-- 回滚参考见 0002_constraints_down.sql (不自动执行)

-- 1. 一个 ticket 仅允许一个处于 active 状态的 listing
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_market_listing_active_ticket
  ON market_listings
(ticket_id) WHERE status = 'active';

-- 2. verification_tokens: ticket + nonce 未使用前唯一
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_verification_ticket_nonce
  ON verification_tokens
(ticket_id, nonce) WHERE used_at IS NULL;

-- 3. events: (organizer_wallet, chain_event_id) 唯一（只针对已有链上 id）
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_events_organizer_chain_id
  ON events
(organizer_wallet, chain_event_id) WHERE chain_event_id IS NOT NULL;

-- 4. trades: 防止重复插入同一交易记录
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_trades_tx_listing
  ON trades
(tx_hash, listing_id);
