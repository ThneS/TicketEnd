-- 0001_init
CREATE TABLE IF NOT EXISTS users (
    wallet TEXT PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'user',
    nonce TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    organizer_wallet TEXT NOT NULL REFERENCES users(wallet),
    chain_event_id BIGINT,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    venue TEXT,
    status TEXT NOT NULL,
    meta JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_types (
    id BIGSERIAL PRIMARY KEY,
    event_id BIGINT NOT NULL REFERENCES events(id),
    price_wei NUMERIC(78,0) NOT NULL,
    supply_total BIGINT NOT NULL,
    supply_sold BIGINT NOT NULL DEFAULT 0,
    meta JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tickets (
    token_id BIGINT PRIMARY KEY,
    event_id BIGINT NOT NULL REFERENCES events(id),
    type_id BIGINT NOT NULL REFERENCES ticket_types(id),
    owner_wallet TEXT NOT NULL,
    seat_label TEXT,
    status TEXT NOT NULL,
    tx_mint TEXT,
    minted_block BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_tickets_owner ON tickets(owner_wallet);
CREATE INDEX IF NOT EXISTS idx_tickets_event ON tickets(event_id);

CREATE TABLE IF NOT EXISTS market_listings (
    id BIGSERIAL PRIMARY KEY,
    ticket_id BIGINT NOT NULL REFERENCES tickets(token_id),
    seller_wallet TEXT NOT NULL,
    price_wei NUMERIC(78,0) NOT NULL,
    side TEXT NOT NULL,
    status TEXT NOT NULL,
    created_block BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_market_listings_status_side ON market_listings(status, side);

CREATE TABLE IF NOT EXISTS trades (
    id BIGSERIAL PRIMARY KEY,
    listing_id BIGINT NOT NULL REFERENCES market_listings(id),
    buyer_wallet TEXT NOT NULL,
    price_wei NUMERIC(78,0) NOT NULL,
    tx_hash TEXT NOT NULL,
    block_number BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_trades_buyer ON trades(buyer_wallet);

CREATE TABLE IF NOT EXISTS verification_tokens (
    id BIGSERIAL PRIMARY KEY,
    ticket_id BIGINT NOT NULL REFERENCES tickets(token_id),
    nonce TEXT NOT NULL,
    qr_code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    verifier_wallet TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_verification_tokens_ticket ON verification_tokens(ticket_id);
CREATE INDEX IF NOT EXISTS idx_verification_tokens_exp ON verification_tokens(expires_at);

CREATE TABLE IF NOT EXISTS blocks_processed (
    id BIGSERIAL PRIMARY KEY,
    chain_id BIGINT NOT NULL,
    last_block BIGINT NOT NULL,
    reorg_marker TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uniq_blocks_processed_chain ON blocks_processed(chain_id);

CREATE TABLE IF NOT EXISTS events_raw (
    id BIGSERIAL PRIMARY KEY,
    block_number BIGINT NOT NULL,
    tx_hash TEXT NOT NULL,
    log_index BIGINT NOT NULL,
    contract TEXT NOT NULL,
    event_sig TEXT NOT NULL,
    data JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uniq_events_raw_identity ON events_raw(tx_hash, log_index);
CREATE INDEX IF NOT EXISTS idx_events_raw_block ON events_raw(block_number);

CREATE TABLE IF NOT EXISTS reconciliations (
    id BIGSERIAL PRIMARY KEY,
    block_number BIGINT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
