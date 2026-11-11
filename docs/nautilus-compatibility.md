# NautilusTrader ↔ CTP 兼容性审查（第一阶段）

> 状态：静态源码审查完成；动态验证未完成。**尚未证实完整兼容**。  
> 约束：绝不修改、Fork、Patch NautilusTrader Core。所有适配必须在本仓库中通过公开扩展接口完成。  
> 上游审查基线：`nautechsystems/nautilus_trader` 的 `develop` 分支（审查日 2026-11-11）。**实施前必须锁定 upstream commit 并重新验证**。

## 1. 总结

NautilusTrader 的 Rust `ExecutionClient`、`HEDGING` OMS、订单事件和启动/持续对账，为独立 CTP 适配器提供了足够的基础抽象。但 Nautilus 通用订单、成交及持仓报告均没有 CTP 专用的 `OffsetFlag`（Open/Close/CloseToday/CloseYesterday）与今昨仓明细字段。上述专有语义需要在 adapter 维护，不能假定它们天然存在于 Nautilus Portfolio。现阶段未证明 `nautilus_ctp` 可以完整且无损地承载所有实盘业务情形。

### 分类

| 领域 | 静态结论 | 遗留验证 |
|---|---|---|
| 期货合约、精度、Tick | 原生抽象可用 | 国内交易所合约编码、动态涨跌停/参数 |
| 行情订阅/转发 | Adapter 可实现 | CTP 行情原始事件与时间戳、断流 |
| K线/夜盘 | Adapter 可实现但有边界 | 夜盘、午休、缺分钟、交易日跨越的聚合语义 |
| 普通限价单、撤单 | Adapter 可实现 | CTP 报单引用和异步回报、拒单竞态 |
| Open/Close/CloseToday/CloseYesterday | Core 报告无专用字段 | 通过订单 tags/适配器专有元数据显式传入；单订单只映射一个 CTP 报单 |
| 今仓/昨仓/冻结仓 | Core 报告无专用字段 | Adapter 内独立的权威持仓账本、冻结、日切 |
| 多空同时持有 | HEDGING 原生支持 | 同合约多空持仓初始化、成交、撤单、对账 |
| 多空启动对账 | 原生支持按 side/venue_position_id 校验 | CTP 无明确 venue_position_id 时的稳定身份映射 |
| 成交补全及合成事件 | 原生机制存在 | 多 side、多 position report 的 partial-window 调整 |
| 账户资金与保证金 | Margin 账户可用 | CTP 权威资金语义、交易所及期货公司加收规则 |
| 重连恢复 | 原生报告接口和 fail-closed 机制 | CTP 查询流控、截断历史、未知报单结果 |
| 原生改单/市价单 | 需 venue-specific 限制 | 不支持则验证拒绝或 cancel-replace，不得虚报能力 |
| 组合/套利合约 | 待验证 | 组合合约/分腿成交/保证金与对账语义 |

## 2. 源码审查证据

1. `crates/common/src/clients/execution.rs`：`ExecutionClient` 公开 `oms_type`、`requires_authoritative_fills`、`retain_unresolved_submissions`、对账容差、bulk position coverage、支持额外 venue 路由与报告方法。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/common/src/clients/execution.rs
2. `crates/model/src/orders/limit.rs`：`LimitOrder` 支持 `tags: Option<Vec<Ustr>>`，但无专用 CTP 开平标志。标签可作为订单意图通道的候选，必须对其经过执行编排、序列化、重启后的可用性做测试。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/model/src/orders/limit.rs
3. `crates/model/src/reports/order.rs`：`OrderStatusReport` 有 side、qty、status、venue_position_id、reduce_only 等，无专用 OffsetFlag。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/model/src/reports/order.rs
4. `crates/model/src/reports/fill.rs`：`FillReport` 有 side、trade_id、commission、venue_position_id，无专用 OffsetFlag。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/model/src/reports/fill.rs
5. `crates/model/src/reports/position.rs`：`PositionStatusReport` 有 account/instrument/side/qty/venue_position_id/avg_px_open，没有 today/yesterday/frozen 字段。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/model/src/reports/position.rs
6. `crates/execution/src/reconciliation/positions.rs` 的 `process_mass_status_for_reconciliation_inner` 中，特定 partial-window fill-adjustment 路径使用某合约的第一条 position report 作为 snapshot；必须针对 CTP 同合约多空并存验证它不会错误合成成交。这不是整个 startup reconciliation 的一般规则。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/execution/src/reconciliation/positions.rs
7. 官方启动对账明确支持 HEDGING 有 venue position ID 时按 ID 核对，且同时出现多空时必须核对两侧总量；不能只看净持仓。  
   https://nautilustrader.io/docs/latest/concepts/execution/reconciliation/
8. 官方 OMS 允许多持仓、策略与 venue OMS 不一致时存在虚拟持仓；虚拟持仓与 CTP 权威持仓不可混同。  
   https://nautilustrader.io/docs/latest/concepts/execution/
9. 官方接入规范要求先实现账户初始化和对账报告，再开放下单，并且需要处理未知结果、回报乱序、重复成交。  
   https://nautilustrader.io/docs/latest/developer_guide/adapters/
10. 官方 Live 文档明确：同 venue 多 execution clients 不代表同 venue 多账户持仓与订单隔离自动完整。  
   https://github.com/nautechsystems/nautilus_trader/blob/develop/docs/concepts/live.md

## 3. 非侵入式实现原则

- `CTP SDK -> C++ boundary -> Rust adapter -> Nautilus public clients/events`。
- CTP 权威账本维度至少保留 Account × Instrument × Direction × Today/Yesterday，并管理平仓冻结及 CTP 报单引用。
- 首版严格要求 **一个 Nautilus Order ↔ 一个 CTP Order**；需跨今昨仓拆分时，先在公开订单编排层生成独立 Nautilus 子订单，再送 Adapter。禁止在 Adapter 偷偷拆分而不对应完整 Nautilus 订单生命周期。
- 指定开平语义必须明确且可恢复；不能用 buy/sell 或 reduce_only 猜测平今和平昨。
- 以 CTP Trade 回报作为成交事实，订单状态、成交 ID 建立稳定去重键。
- 账户可用资金、保证金与真实持仓以 CTP 为权威；Nautilus Portfolio 是交易/策略模型，不等同 CTP 今昨仓账本。
- 若公开接口无法安全表达某条语义，明确记录 **不支持 / 需要外部编排 / fail-closed**，不得修改 Core 或静默改变语义。

## 4. 必须完成的动态验证（验收用例）

| ID | 测试 | 必须成立的断言 |
|---|---|---|
| P01 | 同合约同时多3手、空2手 | Nautilus 两侧持仓都存在，且对账不净额抵消 |
| P02 | 多3手今仓+多2手昨仓 | CTP 账本能精确表示；Nautilus 暴露总量不产生重复 |
| P03 | SHFE/INE 平今/平昨 | 所有下单有准确 OffsetFlag，冻结和可平量正确 |
| P04 | 大连/郑州/中金/广期所开平 | 依据当时交易所规则映射合法 OffsetFlag |
| P05 | 单意图需要拆两笔平仓单 | 两个 Nautilus 子订单身份、成交、撤单均独立 |
| P06 | CTP OnRtnTrade 先于 OnRtnOrder | 最终订单与成交状态一致 |
| P07 | 同一 Trade 重复回报、重启后重放 | 成交只记一次 |
| P08 | 报单发出后断网且结果未知 | 不重复下单，不错误本地终结；恢复后对账 |
| P09 | 同合约多空持仓启动对账 | exact side 与数量校验；无误合成反向成交 |
| P10 | 同合约多空 + 有限成交历史 + avg open | 验证 partial-window fill adjustment 不错误选择第一条报告 |
| P11 | 夜盘跨交易日、周末、节假日 | TradingDay/ActionDay/timestamps/Bar boundaries 正确 |
| P12 | 今昨仓日切及结算 | 仓位 bucket 与费用不重复变化 |
| P13 | 多策略同账户同合约反向仓位 | 真实仓位、虚拟仓位、执行及对账明确可追溯 |
| P14 | 报单、持仓查询流控及失败 | 限流、重试和 fail-closed |
| P15 | 断线期间外部手工下单 | 外部订单及仓位纳入可恢复状态 |
| P16 | CTP 委托费用与保证金变化 | 不依赖固定费率伪造真实账户余额 |
| P17 | 没有今昨仓或 offset 信息的策略平仓 | 明确拒绝或使用显式定义的解析规则，绝不静默误下单 |
| P18 | 多账户同 venue | 没有账户穿透；若核心不能保证隔离，限制部署拓扑 |
| P19 | 无原生改单与市价支持 | 能力声明准确、拒绝或独立实现受控替代 |
| P20 | 模拟重启后标签与执行引用恢复 | OffsetFlag、CTP 订单身份与去重状态持久保留 |

## 5. 验收门槛

阶段一只能判定为 **Conditional / Not yet proven**。完成所有核心执行与恢复用例，且不存在只能通过修改 Core 才能解决的 P0 语义缺口后，才能批准完整适配。否则明确列出受限功能和适用部署模式，不应宣称“完整支持”。

后续按此文档逐项完成源码级论证、确定数据模型、编写 mock fixtures 和 CTP 模拟环境联调；每项记录上游 commit、测试代码、实际结果。
