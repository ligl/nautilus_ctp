# CTP trading sessions and bar aggregation — P0 compatibility audit

> Status: source review complete for basic Nautilus aggregation boundaries; runtime integration UNVERIFIED.  
> Upstream baseline: `nautechsystems/nautilus_trader@4fe0e3c3b60f05b31f8682f24609d5914ddd4267`.  
> Hard rule: **NautilusTrader Core MUST NOT be modified/forked/patched**.

## 1. Terminology: two different session domains

- **TradingSession**: time windows in which a contract can trade; several disjoint day sessions plus optional evening/night session; contract-specific, exchange-specific, timezone Asia/Shanghai, holiday and special-calendar aware.
- **CtpConnectionSession**: CTP MdApi/TraderApi front connections and login identities. `FrontID`, `SessionID`, `OrderRef` and related exchange identifiers must be tracked independently of trading windows. MD and Trader may be separate sessions, have different reconnection behavior, and CTP front switching may change identifiers.
- **TradingDay**: exchange-defined business date; NOT the same as local civil calendar date and NOT the same as a CTP network session. Do not infer solely from UTC event timestamp.

## 2. Confirmed upstream capabilities and limitations

- `DataEngineConfig` includes `time_bars_build_with_no_updates` (default **true**), `time_bars_timestamp_on_close`, `time_bars_skip_first_non_full_bar`, `time_bars_interval_type`, `time_bars_build_delay`, `time_bars_origin_offset`, and `time_bars_time_zone`. Source: https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/data/src/engine/config.rs
- Nautilus supports INTERNAL tick/quote/bar aggregation, including bar-to-bar derived bars. Source: https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/docs/concepts/data/index.md
- Official docs explicitly say: calendar boundaries do not include exchange sessions, holidays or early closes; smaller-than-day periods keep UTC alignment. A timezone for daily bars does **not** imply exchange-session-aware daily bars.
- Therefore, **there is no proof** that Nautilus internal minute/hour/day aggregators satisfy Chinese futures trading-session rules. Particularly, no-update bar generation default true may generate bars over recesses, and a fixed origin offset cannot encode varying contract sessions or holidays.
- This is an identified semantic compatibility gap for using the standard INTERNAL time-bar aggregator **as-is**, not proof of incompatibility of Nautilus Core as an event consumer. A separate adapter-level session-aware aggregator emitting standard `Bar` is a candidate design that requires public pipeline and backtest/live equivalence verification.

## 3. Domain rules to settle before implementation

1. Per-exchange/product/instrument session schedule, including variable night closing times, no-night contracts, suspended holidays and abnormal closings.
2. TradingDay assignment, Friday night/weekend/holiday gaps, `ActionDay` and `TradingDay` from CTP SDK, and the fallback to an authoritative exchange calendar if fields are unavailable/inconsistent.
3. Bar boundary convention: whether bar start/end is anchored to wall clock or trading session open; specify separately for 1m/5m/15m/1h/4h/day.
4. Partial bars at recess, day close, or night close: emit shorter final bar, discard, or defer, per clearly defined bar policy. **Never silently join two disjoint sessions into one time bar** unless the requested bar specification explicitly defines a trading-time aggregation policy.
5. Empty bars: whether no-update bars are suppressed, and whether a sparse session should produce zero-volume carry-forward bars. Must match explicit policy and data source.
6. Bar timestamps: `ts_event` semantic, `ts_init`, close inclusivity, `BarIntervalType`, event-time vs arrival-time skew, no-lookahead.
7. Source CTP `DepthMarketData` is not necessarily a genuine individual trade stream. If producing LAST/volume bars from snapshots, document the approximation and prove how per-tick volume delta, resets, negative corrections and duplicate/out-of-order snapshots are handled. Do **not** call an inferred tick a fully authoritative execution.
8. Bar lifecycle during missing tick sequences, CTP reconnection, duplicated last snapshot, and backfilled historical data. Do not repeat volume after reconnect.
9. Contract differences: historical changes of exchange sessions and contract-specific product hours; do not hardcode a single global session calendar.
10. Backtest/live reproducibility: identical canonical input event sequence and calendar/policy produces equal bar OHLCV, boundaries, and timestamps.

## 4. P0 runtime acceptance matrix (unexecuted)

| ID | Scenario | Required assertion |
|---|---|---|
| BAR-P0-01 | 09:00–10:15, 10:30–11:30 with 15m or 1h aggregation | Policy-defined bar close at recess; no accidentally merged prices/volume |
| BAR-P0-02 | Lunch and 15:00 day close | No spurious no-update bars outside trade windows |
| BAR-P0-03 | Night 21:00 start, different night closing times by contract | Accurate product-specific start/end and local business day |
| BAR-P0-04 | Friday evening, weekend/holiday, first next trading day | Exact trading day assignment; no continuous calendar-day assumption |
| BAR-P0-05 | Day bar across evening/day sessions | Includes market-defined trading day, not midnight cutoff |
| BAR-P0-06 | 1m→5m→15m/1h bar-to-bar chains | Same OHLCV as direct aggregation for equivalent policy and data |
| BAR-P0-07 | Feed reconnect, late tick, duplicate snapshot, volume counter resets | Monotonic, idempotent market volume; explicit late event policy |
| BAR-P0-08 | No ticks for whole interval/session | Explicit empty-bar policy; no synthetic phantom market activity |
| BAR-P0-09 | Tick timestamp / event timestamp boundary equality | Single inclusion, no lookahead, deterministic in replay |
| BAR-P0-10 | Same input in backtest and live mode | Equal canonical Bar stream |

| ID | CTP connection session scenario | Required assertion |
|---|---|---|
| SESSION-P0-01 | MdApi disconnected while TraderApi stays active | MD subscriptions restored; execution state not erroneously reset |
| SESSION-P0-02 | TraderApi reconnect with different FrontID/SessionID | Existing orders correlate via persisted stable identity and venue proof |
| SESSION-P0-03 | OrderRef reuse across trading days/sessions | No ID collision or duplicate execution |
| SESSION-P0-04 | Front failover with unknown outbound order result | No blind resubmit; reconcile or fail-closed |
| SESSION-P0-05 | Query throttling during login and reconnect | Bounded requests, staged account/order/trade/position restoration |
| SESSION-P0-06 | Disconnect across trading day settlement | Correct re-login, roll and reconciliation of today/yesterday positions |
| SESSION-P0-07 | Multiple simultaneous market subscriptions | No duplicate ticks or lost subscription ownership across reconnect |
| SESSION-P0-08 | If multiple TraderApi accounts/sessions enabled | Isolation of account, identity, callbacks, risk state; otherwise explicitly prohibit topology |

## 5. Proposed external-only components

- `CtpTradingCalendar`: exchange/product/contract trading schedule and business-date resolution.
- `CtpSessionAwareBarAggregator`: standard Nautilus `Bar` publisher; explicit policy for boundary/partial/empty bars; no upstream Core changes.
- `CtpMarketDataNormalizer`: timestamp normalization, cumulative-volume delta, dedupe, resets; snapshot-derived bars are labeled/documented accurately.
- `CtpMdSessionManager`: independent MdApi lifecycle, resubscribe and feed continuity.
- `CtpTraderSessionManager`: separate TraderApi lifecycle, auth/login, query throttling and recovery, persistent session/order identity mapping.

### Decision gate

Do **not** implement custom aggregation merely because the market has sessions. First run the above fixtures against the pinned Nautilus INTERNAL aggregator and determine which intervals/policies it supports. Use built-in aggregation where correct; otherwise use the public external-Bar path. This guarantees maximum reuse while preserving market semantics.

Fail any P0 fixture rather than silently treating off-session timestamps as valid. Runtime results, test commands, and input/output fixtures must be committed here before a 'COMPATIBLE' claim.
