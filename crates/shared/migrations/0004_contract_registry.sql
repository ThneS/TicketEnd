-- 0004_contract_registry (PostgreSQL)
-- 合约地址注册表：支持多链、多合约名称，供服务动态加载

CREATE TABLE
IF NOT EXISTS contract_registry
(
    chain_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW
(),
    PRIMARY KEY
(chain_id, name)
);

CREATE INDEX
IF NOT EXISTS idx_contract_registry_chain ON contract_registry
(chain_id);

-- 示例插入（请在生产部署后更新真实地址）
-- INSERT INTO contract_registry(chain_id, name, address) VALUES (11155111, 'TicketManager', '0x...') ON CONFLICT DO NOTHING;
