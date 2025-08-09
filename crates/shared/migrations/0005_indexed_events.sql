-- 0005_indexed_events
-- 链上事件索引持久化表

CREATE TABLE IF NOT EXISTS chain_logs (
    chain_id BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash TEXT NOT NULL,
    log_index INT NOT NULL,
    primary_topic TEXT NOT NULL,
    contract_address TEXT NOT NULL,
    -- 归档原始数据（可选压缩后续）
    data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(chain_id, tx_hash, log_index)
);

-- Ticket 铸造与转移 (最简字段)
CREATE TABLE IF NOT EXISTS ticket_tokens (
    chain_id BIGINT NOT NULL,
    token_id NUMERIC(78,0) NOT NULL,
    event_id NUMERIC(78,0),
    owner TEXT,
    minted_block BIGINT,
    updated_block BIGINT,
    PRIMARY KEY(chain_id, token_id)
);

-- Listing 状态
CREATE TABLE IF NOT EXISTS marketplace_listings (
    chain_id BIGINT NOT NULL,
    listing_id NUMERIC(78,0) NOT NULL,
    ticket_id NUMERIC(78,0),
    seller TEXT,
    status SMALLINT,
    price NUMERIC(78,0),
    created_block BIGINT,
    updated_block BIGINT,
    PRIMARY KEY(chain_id, listing_id)
);

CREATE TABLE IF NOT EXISTS marketplace_trades (
    chain_id BIGINT NOT NULL,
    listing_id NUMERIC(78,0) NOT NULL,
    buyer TEXT,
    price NUMERIC(78,0),
    block_number BIGINT,
    tx_hash TEXT NOT NULL,
    PRIMARY KEY(chain_id, listing_id, tx_hash)
);

CREATE TABLE IF NOT EXISTS token_swaps (
    chain_id BIGINT NOT NULL,
    tx_hash TEXT NOT NULL,
    user_addr TEXT,
    token_in TEXT,
    token_out TEXT,
    amount_in NUMERIC(78,0),
    amount_out NUMERIC(78,0),
    block_number BIGINT,
    PRIMARY KEY(chain_id, tx_hash)
);
