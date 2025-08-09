# OnlineTicket 后端开发计划（Rust + Alloy）

## 1. 总体节奏

周期：~8 周（与合约/前端部分可交叉）。后端聚焦 Indexer、API、验票。

阶段划分：

1. Week 1-2 基础框架与核心 Domain + 数据库迁移
2. Week 2-3 Indexer（回填 + 实时）
3. Week 3-4 API v1（活动、门票、市场、认证）
4. Week 4-5 验票子系统 + 安全加固
5. Week 5-6 Swap 辅助接口 + 性能优化 + 指标
6. Week 6-7 集成测试 + Reorg/混沌演练
7. Week 7-8 压测、观察、打包上线

## 2. 任务拆分 (WBS)

### Week 1-2 基础设施 & Domain

- [ ] 初始化 workspace (Cargo workspaces: shared, api, indexer, verifier)
- [ ] 引入依赖 (axum, tracing, serde, sqlx, redis, alloy, anyhow, thiserror, jsonwebtoken)
- [ ] 配置管理模块 (config.rs)
- [ ] 日志 + OpenTelemetry 初始化
- [ ] 数据库 schema 设计 & 初始迁移 (users/events/tickets/...)
- [ ] Domain 模型（Event, Ticket, User, Listing 等）定义
- [ ] 错误枚举与统一响应中间件
- [ ] SIWE Challenge / Nonce 生成逻辑 (伪实现)

验收标准：

- `cargo test` 通过
- `sqlx migrate run` 成功建立基础表
- GET /health 返回 200

### Week 2-3 Indexer

- [ ] 引入 Alloy provider (HTTP + WS)
- [ ] ABI 绑定生成脚本 (build.rs 或 xtask)
- [ ] 回填任务：按区块区间抓取 logs 写入 events_raw
- [ ] 事件解码 + 投影：写 tickets/events/listings
- [ ] 实时订阅：WS + 自动重连
- [ ] Reorg 检测：比较 block hash -> 回滚重放策略
- [ ] 指标：indexer_last_block / chain_lag
- [ ] 集成测试：Anvil 环境中模拟 Mint/Transfer

验收标准：

- 模拟 2000 区块回填 < 合理时间（<60s）
- Reorg 2 区块内正确恢复
- 指标端点 /metrics 暴露数据

### Week 3-4 API v1

- [ ] 路由结构搭建 (events, tickets, market, auth)
- [ ] 活动查询 /events /events/{id}
- [ ] 票种查询 /events/{id}/tickets/types
- [ ] 门票详情 /tickets/{token_id}
- [ ] 市场列表 /market/listings
- [ ] 登录流程：/auth/siwe/challenge /auth/siwe/verify -> JWT
- [ ] 组织者创建活动（链下草稿 + 链上发售前准备）
- [ ] 列表分页、排序、过滤
- [ ] OpenAPI 文档 (utoipa or okapi)
- [ ] 基本速率限制中间件

验收标准：

- OpenAPI 文档可访问
- 关键接口 P95 < 150ms（本地）
- JWT 认证成功保护受限路由

### Week 4-5 验票子系统

- [ ] QR Token 生成接口 /tickets/{id}/qrcode（需认证）
- [ ] HMAC 方案 + Redis 存储（过期 + 单次）
- [ ] 扫码接口 /verify/scan（Verifier Key）
- [ ] Ticket 状态原子更新 + 冪等保证
- [ ] 并发场景测试（同票多次扫描只有一次成功）
- [ ] 指标：qr_verify_latency_ms, ticket_verify_conflicts
- [ ] 审计日志：记录核销（who, when, where）

验收标准：

- 并发 50 并行扫描，成功 1 其余返回已使用
- 平均验证接口耗时 < 80ms（本地）

### Week 5-6 Swap 支撑 & 优化

- [ ] Swap quote 接口（调用合约 getReserves / 计算 amountOut）
- [ ] Slippage 校验参数
- [ ] 缓存层（报价短 TTL）
- [ ] Multicall 批量获取多交易对储备
- [ ] 性能 profiling（flamegraph）
- [ ] DB 查询优化（索引、EXPLAIN 分析）

验收标准：

- 单次报价 < 50ms（缓存命中 <10ms）
- 多交易对批量接口正确返回

### Week 6-7 集成 & 混沌

- [ ] 端到端测试：购票（模拟上链） -> 索引 -> API 查询 -> 生成二维码 -> 核销
- [ ] Reorg 混沌脚本：回滚 N 区块
- [ ] 回滚后数据一致性校验脚本
- [ ] 安全测试：重复 nonce、过期 token、JWT 伪造
- [ ] 资源压力测试：CPU、内存、连接池耗尽

验收标准：

- 端到端脚本 100% 通过
- 混沌回滚后数据一致
- 无高危安全漏洞（初步扫描）

### Week 7-8 上线准备

- [ ] Dockerfile 多阶段构建 + 镜像扫描
- [ ] Helm Chart / docker-compose 样例
- [ ] 配置分环境文件 (.env.example)
- [ ] 日志/指标仪表盘 (Grafana JSON)
- [ ] 警报规则：indexer lag > 120s, error_rate > 2%
- [ ] 运行手册 / 运维文档
- [ ] 最终性能压测报告

验收标准：

- 镜像大小 < 150MB
- 指标+日志仪表盘可用
- 压测：QPS 500 下错误率 < 1%

## 3. 角色与职责 (示例)

| 角色         | 职责                             |
| ------------ | -------------------------------- |
| 后端工程师 A | Indexer + Reorg + Multicall      |
| 后端工程师 B | API 路由 + 验票服务              |
| 全栈 / 协调  | Swap 支撑 + OpenAPI + 部署流水线 |
| QA           | 测试方案 / 混沌脚本              |

## 4. 依赖关系

- Indexer 完成（至少基础）后，API 才能提供实时票与市场数据
- 验票依赖 tickets.status 正确投影
- Swap 接口依赖合约部署地址稳定
- 混沌测试依赖 Reorg 处理

## 5. 里程碑

| 里程碑         | 时间  | 可交付物                  |
| -------------- | ----- | ------------------------- |
| M1 基础框架    | W2 末 | schema + health + domain  |
| M2 Indexer MVP | W3 末 | 回填+实时+投影+指标       |
| M3 API v1      | W4 末 | OpenAPI + 核心查询 + 认证 |
| M4 验票完成    | W5 末 | QR 流程 + 并发保障        |
| M5 Swap & 优化 | W6 末 | 报价+缓存+性能报告        |
| M6 集成稳定    | W7 末 | 端到端 + 混沌通过         |
| M7 上线准备    | W8 末 | 镜像+脚本+仪表盘          |

## 6. 风险与应对

| 风险               | 等级 | 应对                                   |
| ------------------ | ---- | -------------------------------------- |
| Reorg 复杂度高     | 高   | 早期实现最小 Reorg 支持 + 单元测试覆盖 |
| Alloy API 变动     | 中   | 锁定版本 + 监控 release note           |
| 性能瓶颈（索引慢） | 中   | 批量抓取+并行分片 + 指标监控调优       |
| 签名欺骗 / 重放    | 高   | SIWE nonce 单次 + exp + Redis 原子操作 |
| 并发验票写冲突     | 中   | 行级锁或乐观更新 + retry               |
| 缓存不一致         | 中   | TTL 短 + 主查询 fallback               |

## 7. 度量指标 (DoD)

- 代码质量：clippy 零警告，fmt 统一
- 测试覆盖：核心 Domain > 85%
- 性能：关键接口 P95 < 150ms
- 稳定性：Index Lag < 10 区块（正常情况下）
- 安全：无 High 级别静态扫描漏洞

## 8. 工具 & 自动化

- Makefile: build/test/run/indexer
- Git Hooks: pre-commit (fmt+clippy)
- CI: lint -> test -> build 镜像 -> 推送 -> 部署
- Seed 脚本：插入 demo 活动/门票

## 9. 验收流程

1. 每周演示（演示环境）
2. PR 审查：>= 1 人 Review
3. Checklist：日志、指标、错误处理、文档
4. Stage 回归 -> Prod 发布

## 10. 后续 Backlog

- GraphQL 层
- Webhook 通知
- zk 验票
- 多链扩展
- AI 推荐（活动个性化）

---

该计划确保在 8 周内逐步交付一个安全、可靠、可扩展的 Rust 后端，为 OnlineTicket 链上生态提供坚实支撑。
