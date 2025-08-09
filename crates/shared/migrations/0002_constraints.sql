-- 0002_constraints
-- 为市场活动添加业务唯一性与约束

-- 1. 一个 ticket 在二级市场同一时间只能有一个处于 active 状态的 listing
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_market_listing_active_ticket
ON market_listings
(ticket_id)
WHERE status = 'active';

-- 2. verification_tokens: 防止同一张票在未失效前生成重复 nonce （可选）
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_verification_ticket_nonce
ON verification_tokens
(ticket_id, nonce)
WHERE used_at IS NULL;

-- 3. events: organizer + chain_event_id（如果链上 ID 已知）唯一
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_events_organizer_chain_id
ON events
(organizer_wallet, chain_event_id)
WHERE chain_event_id IS NOT NULL;

-- 4. trades: 防止重复写入同一交易（tx_hash + listing_id）
CREATE UNIQUE INDEX
IF NOT EXISTS uniq_trades_tx_listing
ON trades
(tx_hash, listing_id);
