# Execution / Position compatibility — source audit 02

Status: **STATICALLY REVIEWED / DYNAMICALLY UNVERIFIED**  
Date: 2026-11-11  
Upstream immutable source revision: `4fe0e3c3b60f05b31f8682f24609d5914ddd4267`  
Hard boundary: **no modification, fork or patches to NautilusTrader Core**.

This report extends [nautilus-compatibility.md](nautilus-compatibility.md). It does **not** assert that integration or runtime tests passed.

## Confirmed public mechanisms

| Requirement | Source | Finding | Remaining uncertainty |
|---|---|---|---|
| Multiple long and short positions | [positions.md](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/docs/concepts/positions.md) | HEDGING tracks multiple positions by distinct IDs; opposite sides are not automatically netted | Stable CTP ID mapping, reconciling historic fills |
| Adapter trading commands and report interfaces | [ExecutionClient](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/common/src/clients/execution.rs) | Public implementation points for order submission/cancel, order/fill/position reports, account states | Exact CTP session timing, query restrictions |
| Open/Close/CloseToday/CloseYesterday payload | [LimitOrder](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/model/src/orders/limit.rs), [OrderInitialized](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/model/src/events/order/initialized.rs), [OrderSnapshot](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/model/src/events/order/snapshot.rs) | Explicit tags exist across order creation/event/snapshot models; CTP offset is not a dedicated core field | Does metadata survive all order routes and restart? |
| Venue-side position details | [PositionStatusReport](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/model/src/reports/position.rs) | Contains side, quantity and optional venue position ID but no today/yesterday or frozen field | Adapter must enforce distinct venue ledger truth |
| Order/fill details | [OrderStatusReport](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/model/src/reports/order.rs), [FillReport](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/model/src/reports/fill.rs) | Do not carry CTP offset dedicated fields; fill carries venue trade ID, price, quantity, commission, position ID | Adapter report transformation, fill identity key |
| Authoritative fills | [ExecutionClient](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/common/src/clients/execution.rs) | `requires_authoritative_fills` avoids inferring fills from order-status updates | Position-only synthetic corrections are NOT prevented by this option |
| Unknown order results | [ExecutionClient](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/common/src/clients/execution.rs) | `retain_unresolved_submissions` signals that only venue evidence can close unresolved submissions | Durable correlation after CTP restart; exhaustion behavior |
| Incomplete position query | [ExecutionClient](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/common/src/clients/execution.rs) | `provides_bulk_position_coverage` can indicate absent report does not establish flat exposure | Correct per-instrument coverage semantics for CTP |
| Partial-window correction | [positions.rs](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/execution/src/reconciliation/positions.rs) | `process_mass_status_for_reconciliation_inner` uses first position report with avg price as correction target in one branch | Two-sided simultaneous position may require conservative reporting or non-generating adjustment; must test |
| Startup two-sided reports | [reconciliation.md](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/docs/concepts/execution/reconciliation.md), [manager.rs](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/live/src/execution/manager.rs) | Position-specific reconciliation available for `venue_position_id`; startup reconciliation should not net simultaneous opposite sides | Check side, number of reports, ID stability and real CTP account baseline |

## P0 compatibility questions, with falsifiable proof criteria

### CTP-P0-01: Position IDs

Proposed contract: one **stable** logical VenuePositionId for each `Account × Instrument × Direction` exposure (or finer identity only when it survives query and restart), not one ID per CTP query row. CTP today's and yesterday's buckets remain an internal authority ledger and sum into corresponding side exposure.

Must test:
- Long today + long yesterday has one correctly aggregated exposure (or several stable externally justified IDs), no duplicates.
- Long and short concurrent: independent non-netted reports and positions.
- Query twice unchanged, daily settlement rolls today→yesterday, restart persists same identity.
- External/manual position changes do not get attributed to unrelated strategies.

### CTP-P0-02: Offset command semantics

Never infer `CloseToday` / `CloseYesterday` from order side, `reduce_only` or net exposure. Explicit `OffsetFlag` must arrive by a **validated public command pathway** or safely resolved from CTP authoritative buckets and exchange rules. Missing/ambiguous intent is an error; do not submit a CTP order. Initial mode: one Nautilus order maps to one CTP order. Splitting occurs before adapter submission, producing independent Nautilus child orders.

Must test:
- Explicit Open, CloseToday, CloseYesterday.
- Across-day split with distinct Nautilus IDs.
- Tags present after new/restart/replay, after execution-algorithm path if used.
- Reject malformed/ambiguous tag; no silent default for unsafe close.
- Available today/yesterday quantities and close freezes accounted for, including in-flight commands.

### CTP-P0-03: Reconciliation correction safety

`requires_authoritative_fills=true` avoids order-status-derived synthetic fills but **does not disable all position-only corrections**. A separate partial-window adjustment path uses a first position report and may synthesize an opening fill when `avg_px_open` is present. Do not assume this is safe for simultaneous long and short in CTP. Verify the whole mass-status preparation and startup flow under HEDGING with two sides.

Must test:
- Two reports for same instrument with stable distinct IDs, both have prices; short fill history; ensure no synthetic fill incorrectly nets or changes opposite side.
- Missing opening history and a single side; check exact regenerated exposure and fees.
- No average price in venue report: check whether correction is deliberately skipped, while still successfully reconciling using authoritative position report (must prove).
- Insufficient proof results in fail-closed startup, not fabricated success.

### CTP-P0-04: Unknown-submit and fill identity

Store CTP request identity/correlations durably: account/broker, trading day, front/session, order reference, exchange/order system identifiers, venue trade identifiers. Exact canonical keys depend on the CTP SDK and broker response; validate them against real fixtures before implementation.

Must test:
- Trade before order, duplicate trade, duplicate order callback.
- Response lost after transport success; restart before resolution; never blindly retransmit.
- Order ref reuse across sessions and trading days; do not collide.
- Reconciliation history shorter than time-to-live; document and preserve sufficient local evidence.
- Query throttled/timeout: retain unknown or block submission rather than assuming rejected.

## Architecture decision pending proof

Proposed external components: `CtpInstrumentProvider`, `CtpMarketDataClient`, `CtpExecutionClient`, `CtpPositionLedger`, `CtpOffsetResolver`, `CtpOrderIdentityStore`, `CtpTradingCalendar`, `CtpReconciliationMapper`.

No adapter implementation authorized as 'validated' until test evidence is supplied. For any P0 failure: either fix entirely within the adapter/public-extension contract, explicitly restrict the unsupported mode, or mark project compatibility blocked. **Never modify NautilusTrader Core**.

## Verification protocol

1. Pin upstream to `4fe0e3c3b60f05b31f8682f24609d5914ddd4267` for current evidence; rerun on the version actually deployed.
2. Build dedicated integration fixtures in **this** repository, invoking Nautilus public components; upstream repository is read-only.
3. Record test command, dependency commit, outcome, and exact expected state for each P0 case.
4. Mark status only after executed tests; static review is not a pass.
