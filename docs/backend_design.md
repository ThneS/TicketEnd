# OnlineTicket 后端设计（Rust + Alloy 版）

## 1. 设计目标

- 为链上门票/活动/市场/代币互换智能合约提供高性能、可观测、可扩展、低耦合的链下支撑服务。
- 提供统一的 REST + WebSocket API（后期可拓展 GraphQL）给前端（用户端、组织者端、核销端）使用。
- 确保链上事件与链下数据库强一致（最终一致 + Reorg 修正能力）。
- 提供安全、低延迟的线下扫码验票能力（防重放、防伪造、防二次使用）。
- 利用 Alloy 实现对合约的高性能调用、事件抓取、批量查询与签名工具链。

## 2. 范围与不做（Scope / Out of Scope）

In Scope:

- 链上合约事件索引/回填/重组处理
- API 服务（活动、门票、市场、互换查询 & 下单辅助）
- 验票服务（生成/验证二维码 Token）
- 用户认证（基于钱包签名 + 组织者角色）
- 任务调度与缓存
- 监控、日志、审计轨迹

Out of Scope（当前版本不实现）:

- 深度 DeFi 聚合路由优化（仅基础 swap 辅助查询）
- 高阶 AML/KYC 流程
- 多链聚合（v1 仅单链，如 Sepolia / 主网）
- 离线批量票券 PDF 生成（后续扩展）

## 3. 技术选型

| 层次       | 方案                                         | 说明                                       |
| ---------- | -------------------------------------------- | ------------------------------------------ |
| 语言       | Rust                                         | 安全 + 性能 + 并发                         |
| Web 框架   | Axum                                         | 现代、Tower 中间件生态完善                 |
| 区块链交互 | Alloy                                        | 统一 Provider / ABI / 事件解码 / multicall |
| 异步运行时 | Tokio                                        | 标准生态                                   |
| 数据库     | PostgreSQL                                   | 事务一致性 + JSONB + 索引丰富              |
| 缓存       | Redis                                        | 热数据与分布式锁、验证码、短期票据         |
| 队列/任务  | Redis Stream / SeaQueue (可选)               | 轻量实现延迟/重试；后期可换 Kafka          |
| 日志       | tracing + OpenTelemetry                      | 结构化日志 + 分布式追踪                    |
| 指标       | Prometheus + Grafana                         | QPS、延迟、滞后高度、失败率                |
| 配置       | figment + dotenvy                            | 分环境加载与覆盖                           |
| 依赖注入   | shaku/手写容器                               | 降低耦合                                   |
| 序列化     | serde / serde_json                           |                                            |
| 测试       | cargo test + k6(性能) + cucumber(行为，可选) |                                            |
| 安全       | JWT + EIP-4361(SIWE) + HMAC(验票)            |                                            |
| 架构风格   | Clean / Hexagonal                            | Domain 隔离 infra                          |

## 4. 模块与服务划分

1. api-gateway（Axum 主服务）
2. indexer（区块 & 事件索引器，可嵌入或独立进程）
3. verifier（验票服务，低延迟接口，可合并到 api 但推荐独立以隔离流量）
4. scheduler / worker（异步任务：回填区块、重试失败事件、生成统计报表）
5. shared crate（domain、models、contracts、error、config、utils）

部署形态：

- monorepo 多 crate；初期单二进程（api + indexer），随负载拆分。

## 5. 目录建议（示例）

```
# 顶层 workspace 结构（已取消 backend 目录）
Cargo.toml
crates/
  shared/
    src/
      config.rs
      error.rs
      db/
      domain/
        event.rs
        ticket.rs
        marketplace.rs
        swap.rs
        user.rs
      contracts/
        abis/ (JSON / auto gen bindings)
        mod.rs
      security/
        auth.rs
        signer.rs
        qr.rs
      utils/
  indexer/
    src/
      main.rs
      pipeline/
        fetch.rs
        decode.rs
        persist.rs
        reconcile.rs
  api/
    src/
      main.rs
      routes/
        events.rs
        tickets.rs
        marketplace.rs
        swap.rs
        user.rs
        verify.rs
      middleware/
      dto/
      services/
      state.rs
  verifier/
    src/
      main.rs
      routes/verify.rs
.env.example
Makefile
LICENSE
README.md (未来可加入)
```

## 6. Domain 概念模型

- Event: 活动（id, organizer, start_time, end_time, venue, status）
- TicketType: 票种（id, event_id, price, supply, reserved）
- TicketNFT: 单张门票（token_id, owner, event_id, seat, status: valid|used|revoked）
- Order / Listing: 市场订单（id, ticket_id, seller, price, side: primary|secondary, status）
- SwapQuote: 代币互换报价（输入 token, 输出 token, amount_in, amount_out, slippage）
- User: 用户/组织者（wallet, role, nonce, profile）
- VerificationToken: 短期验票凭证（ticket_id, nonce, exp, signature/hmac）

## 7. 数据库设计（简化 ERD）

```
users (wallet PK, role, nonce, created_at)
events (id PK, organizer_wallet FK->users, chain_event_id?, start_time, end_time, venue, status, meta JSONB)
ticket_types (id PK, event_id FK, price_wei, supply_total, supply_sold, meta JSONB)
tickets (token_id PK, event_id FK, type_id FK, owner_wallet, seat_label, status, tx_mint, minted_block)
market_listings (id PK, ticket_id FK, seller_wallet, price_wei, side, status, created_block)
trades (id PK, listing_id FK, buyer_wallet, price_wei, tx_hash, block_number)
swap_quotes_cache (hash PK, payload JSONB, created_at, ttl)
verification_tokens (id PK, ticket_id FK, nonce, qr_code_hash, expires_at, used_at, verifier_wallet)
blocks_processed (id serial, chain_id, last_block, reorg_marker)
events_raw (id serial, block_number, tx_hash, log_index, contract, event_sig, data JSONB, processed BOOL)
reconciliations (id serial, block_number, status, created_at)
```

索引：

- tickets(owner_wallet), tickets(event_id), market_listings(status, side), trades(buyer_wallet)
- events(start_time), verification_tokens(ticket_id, expires_at)
- events_raw(block_number, contract), events_raw(tx_hash)

## 8. 与智能合约交互 (Alloy)

- 生成 ABI Bindings：在 `build.rs` 中读取 `abis/*.json` 用 alloy bind 生成类型安全调用。
- Provider：Http + WebSocket（WS 用于实时日志）。
- Event 流：ws 订阅 + 回退容忍策略（检测 block hash 变化 -> 标记受影响区块 dirty -> 重放）。
- 批量查询：multicall 获取批量 ticket metadata / ownerOf / listing 数据。
- Gas 估算 & 模拟：对需要链上交易前的辅助接口（如“预估购买”）。
- 重试策略：指数退避 + 最大 N 次 + 死信记录。

## 9. 事件索引流程

1. 启动时读取 `blocks_processed`，确定起始高度（或创世配置）。
2. Backfill：分批（如 500~2000 区块）抓取 logs -> 解码 -> 入 `events_raw` -> 业务投影写入主表。
3. 实时：WS 订阅 -> 解码 -> 直接投影 & 写 raw。
4. Reorg：检测 block hash 不一致 -> 回滚该高度及之后受影响记录（按 events_raw 反向操作 / 状态重建）。
5. 幂等：投影层使用 UPSERT + 事件 (tx_hash, log_index) 唯一约束。
6. 健康：指标（滞后高度=链头-已处理高度）。

## 10. 验票与二维码安全

流程：

1. 用户在前端请求生成二维码：API 生成 `verification_token`（包含 ticket_id, 随机 nonce, exp）
2. 服务器生成 payload：`ticket_id | nonce | exp` -> HMAC_SHA256(secret) -> 输出 base64
3. 前端展示二维码(加密 JSON 或紧凑 base64url)
4. 线下核销 App 扫码 -> 调后端 `/verify/scan`：
   - 校验 exp, nonce 未用 & 未过期
   - 查询 ticket 状态链下缓存 + 可选链上 owner 校验（可异步快速返回 + 后台补充）
   - 标记 verification_token.used_at + tickets.status = used（可选：写链上“consume”交易或者使用合约内状态位）
5. 防重放：nonce 单次使用 + tickets.status 切换
6. 可选双通道：快速本地缓存判定 + 延迟链上确认（补偿写）

## 11. 认证与授权

- 用户登录：SIWE（EIP-4361）签名 -> 校验后发 JWT（短） + Refresh Token（Redis 存储，旋转）
- 组织者角色：`users.role in ('organizer','admin')`
- 管理操作（创建活动、发售）需校验其链上权限（合约 organizer 列表 / role registry）缓存 5 分钟。
- 验票终端：发行“Verifier API Key” (HMAC) + 限定 IP / 速率。

## 12. API 设计（摘要）

Public:

- GET /health
- GET /events?status=&page=
- GET /events/{id}
- GET /events/{id}/tickets/types
- GET /tickets/{token_id}
- GET /market/listings?event_id=&status=&side=
- GET /market/listings/{id}
- GET /swap/quote?in_token=&out_token=&amount=

Auth (User):

- POST /auth/siwe/challenge
- POST /auth/siwe/verify
- POST /tickets/{token_id}/qrcode (生成验票二维码)

Organizer/Admin:

- POST /events
- POST /events/{id}/ticket-types
- POST /market/listings (primary issue / secondary)
- POST /market/listings/{id}/cancel

Verifier:

- POST /verify/scan

## 13. 缓存策略

- 热门活动列表：Redis key events:hot:page:\* 60s
- Swap 报价：短 TTL（5~15s），key hash by pair+amount bucket
- SIWE Nonce：nonce:{wallet} 5 分钟
- Contract metadata / roles：role:{wallet} 300s
- Verification token：vt:{id} 到期自动失效（EXPIRE）

## 14. 性能与扩展

- Axum + Tokio：使用结构化路由 + 分层中间件（日志、限流、认证）。
- 数据库连接池：deadpool-postgres / bb8，基于负载调优（max_connections ~ 50）。
- Indexer 与 API 分离线程池，防止阻塞。
- 批处理：事件 decode 后批量 INSERT + ON CONFLICT。
- 读多写少：前端查询尽量命中缓存 + 分页 SQL 索引。
- 水平扩展：
  - API 无状态（sessionless JWT） -> K8s 扩副本
  - Indexer 单主（leader election，通过 Postgres advisory lock）
  - 读副本（只读查询）后期添加

## 15. 安全设计

- 输入校验：DTO 层 + type-safe NewType（WalletAddress 等）。
- 速率限制：基于 tower-ratelimit + Redis 令牌桶。
- HSTS + TLS（由 ingress 层处理）。
- 敏感日志脱敏（钱包地址仅展示前后 6 位可选）。
- 重放防护：SIWE nonce 单次消费；二维码 nonce 单次消费。
- 签名验证：Alloy 提供 EIP-191/EIP-712 支持。
- 依赖审计：cargo deny, cargo audit。

## 16. 观测与可运维性

- tracing span：request_id, user_wallet, route, db_time, chain_lag。
- Prometheus Metrics：
  - http_requests_total{route,status}
  - http_request_duration_seconds
  - indexer_last_block
  - indexer_chain_lag
  - events_decode_errors_total
  - qr_verify_latency_ms
- 日志等级：info (业务), warn (潜在问题), error (失败), debug (临时)。
- 健康检查：/health（DB, Redis, Indexer 滞后 < 阈值）。

## 17. 配置与密钥管理

- 多 profile：dev / staging / prod
- 环境变量：`RPC_HTTP_URL`, `RPC_WS_URL`, `DB_URL`, `REDIS_URL`, `JWT_SECRET`, `QR_HMAC_SECRET`
- Secrets 存储：K8s Secret 或 Vault。
- 动态可热更新（非关键项）通过 Redis PubSub 广播。

## 18. 错误处理与返回规范

- 统一错误枚举：DomainError / InfraError / AuthError。
- HTTP 映射：400(参数), 401(未认证), 403(权限), 404(不存在), 409(状态冲突), 422(业务规则), 500(内部)。
- 错误响应：`{ "error_code": "TICKET_ALREADY_USED", "message": "该门票已核销" }`

## 19. 测试策略

- 单元：domain 纯逻辑（无 IO）100% 覆盖关键路径。
- 集成：使用 testcontainers 拉起 Postgres + Redis + anvil，本地模拟链事件。
- 合约交互：利用 Anvil + Alloy，脚本 mint/transfer 测试索引器。
- 性能：k6 针对购票、列表、verify 场景（P95 < 120ms）。
- 混沌：模拟 Reorg（Anvil 提供）验证回滚逻辑。

## 20. 部署流水线

- GitHub Actions：
  - lint (clippy + fmt)
  - security (cargo audit)
  - test (unit + integration)
  - build (docker image, tag with git sha)
  - deploy (staging -> manual promote -> prod)
- 镜像：多阶段构建（builder + distroless/ubi minimal）

## 21. 迁移与版本

- DB 迁移：sqlx migrate 或 refinery。
- 合约地址变动：`contracts_registry` 表 + 配置文件，可通过版本号切换；Indexer 支持多合约源。
- 版本策略：SemVer；API 通过 /v1 前缀。

## 22. 未来扩展预留

- GraphQL 网关（活动/门票聚合）
- 多链支持：增加 chain_id 维度 / 分表
- Webhook 通知（票卖出、活动更新）
- AI 推荐：消费行为 + embeddings
- 支持 zk 证明门票持有（隐私核销）

## 23. 风险与缓解

| 风险                     | 描述           | 缓解                                       |
| ------------------------ | -------------- | ------------------------------------------ |
| Reorg 影响一致性         | 状态回放错误   | 事件 raw + 幂等投影 + 哈希校验             |
| WS 断线                  | 实时丢事件     | 定期补扫区块范围重放                       |
| 高并发验票               | 热点写锁冲突   | Redis 短缓存 + 乐观更新 + 队列缓冲         |
| Slippage 误差            | 报价与实际不同 | 返回报价时附上有效期与最小可接受输出       |
| 私钥泄露（若服务器代签） | 资产风险       | 最小化服务器签名用途, 使用硬件签名或不托管 |

## 24. 示例关键代码片段（概念性）

```rust
// 事件订阅（简化）
async fn stream_events(provider: WsProvider, filters: Vec<LogFilter>) {
    let mut sub = provider.subscribe_logs(&filters).await.unwrap();
    while let Some(log) = sub.next().await {
        match decode_event(log) { Ok(evt) => handle(evt).await, Err(e) => warn!(?e); }
    }
}

// 验票 Token 生成
fn generate_qr(ticket_id: i64, secret: &[u8], ttl: Duration) -> QrPayload { /* ... */ }
```

## 25. 结论

该设计在保证链上可信前提下提供高性能链下支撑，采用事件溯源 + 幂等投影 + Alloy 强类型交互，满足初始版本快速上线与后期扩展需要。
