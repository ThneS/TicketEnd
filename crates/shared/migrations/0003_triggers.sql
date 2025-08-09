-- 0003_triggers
-- 通用 updated_at 触发器 (PostgreSQL)

CREATE OR REPLACE FUNCTION set_updated_at
()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW
();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 为需要 updated_at 字段的表添加列 & 触发器（若不存在）
ALTER TABLE events ADD COLUMN
IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW
();
ALTER TABLE tickets ADD COLUMN
IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW
();
ALTER TABLE market_listings ADD COLUMN
IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW
();
ALTER TABLE trades ADD COLUMN
IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW
();

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1
    FROM pg_trigger
    WHERE tgname = 'trg_events_updated_at') THEN
    CREATE TRIGGER trg_events_updated_at BEFORE
    UPDATE ON events FOR EACH ROW
    EXECUTE PROCEDURE set_updated_at
    ();
END
IF;
  IF NOT EXISTS (SELECT 1
FROM pg_trigger
WHERE tgname = 'trg_tickets_updated_at') THEN
CREATE TRIGGER trg_tickets_updated_at BEFORE
UPDATE ON tickets FOR EACH ROW
EXECUTE PROCEDURE set_updated_at
();
END
IF;
  IF NOT EXISTS (SELECT 1
FROM pg_trigger
WHERE tgname = 'trg_market_listings_updated_at') THEN
CREATE TRIGGER trg_market_listings_updated_at BEFORE
UPDATE ON market_listings FOR EACH ROW
EXECUTE PROCEDURE set_updated_at
();
END
IF;
  IF NOT EXISTS (SELECT 1
FROM pg_trigger
WHERE tgname = 'trg_trades_updated_at') THEN
CREATE TRIGGER trg_trades_updated_at BEFORE
UPDATE ON trades FOR EACH ROW
EXECUTE PROCEDURE set_updated_at
();
END
IF;
END $$;
