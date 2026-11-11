# Official Adapter Integration Contract — Pure Rust

Date: 2026-11-11  
Status: revised; supersedes all Python/PyO3 proposals.

## Non-negotiable architecture

1. The entire `nautilus_ctp` implementation is **Rust only**. No Python interface, Python façade, PyO3, or Python runtime.
2. All source code, documents, binaries and tests live in independent `ligl/nautilus_ctp`.
3. **Never modify, fork or patch NautilusTrader Core.**
4. Reuse the upstream Nautilus Rust crates and public client traits. Pin all Nautilus crates to the **same immutable upstream revision or compatible published version**.
5. Prove compilation and real Rust `LiveNode` registration using a standalone Cargo workspace. Do not claim success from existence of traits alone.

## Official references and correct interpretation

- [Developer guide: Adapters](https://nautilustrader.io/docs/latest/developer_guide/adapters/) describes *in-tree* adapter crate wiring and optional Python projections. Its instructions to modify the upstream Rust workspace or PyO3 registry **do not apply**.
- [Concepts: Rust](https://nautilustrader.io/docs/latest/concepts/rust/) confirms a complete pure-Rust engine, Rust `LiveNode`, independent Cargo binary, and Nautilus crates on crates.io; hence Python is **not** a required runtime or integration boundary.
- [Rust live trading how-to](https://nautilustrader.io/docs/latest/how_to/live_trading_rust/) offers Rust builder patterns (exact location and compatibility must be checked against pinned source).
- Out-of-tree direct Rust adapter registration is a **compatibility test to perform**: build adapter crate implementing public `DataClient`/`ExecutionClient` and corresponding factory traits, then register via public `LiveNode` builder/factory APIs in a standalone executable. The official Rust docs support independently compiled Rust applications; they do not automatically prove every intended CTP factory signature is public and stable.

## Proposed standalone architecture (subject to compile proof)

```text
ligl/nautilus_ctp (independent Cargo workspace)
  crates/ctp-sys                 CTP native C++ FFI
  crates/nautilus-ctp            Rust data/execution adapter, session manager,
                                 position/offset ledger, trading calendar, bar handling
  examples/ctp-live-node        Standalone Rust binary owns Nautilus LiveNode
  tests/                        Pure Rust integration and deterministic fixtures
                     |
                     v
    nautilus-live + nautilus-data + nautilus-execution
    nautilus-common + nautilus-model (unmodified crates)
```

CTP C++ SDK at the FFI boundary is unavoidable external SDK code and **does not introduce Python into the implementation**.

## Stage 0 proof of independent Rust integration

- Instantiate an adapter through public Rust interfaces without vendoring/modifying Nautilus.
- Add it to a Rust `LiveNode` via public builder/factory interface, start and stop without CTP SDK (mock transport).
- Deliver a typed `QuoteTick` and optionally `Bar` to `DataEngine` and a Rust strategy subscriber.
- Deliver a controlled `OrderAccepted`, `OrderFilled`, and account report through the public execution path; verify Engine/Cache.
- Verify same lockfile/revision and no duplicate versions of Nautilus types.
- Verify restart and persistence limitations **in the native Rust deployment**. Prior claimed Python-client Redis/PostgreSQL restriction is inapplicable and must not be carried into the Rust-only design.
- Run tests and capture commands/output. Until executed, the result remains **UNVERIFIED**.

## CTP-specific P0 tests retained

- Trading-session-aware K-line aggregation: irregular morning/lunch/night sessions, holiday, trading day, no-update bars, volume counter reset.
- MD and Trader connection sessions must be modeled independently.
- HEDGING and simultaneous long/short, today/yesterday split, explicit CTP OffsetFlag, order identity, duplicate fills, disconnection and fail-closed reconciliation.

## Risk statement

**Supported at a product level:** Rust NautilusTrader runtime, Rust LiveNode, public Rust crates and standalone binaries.  
**Still needs compile/integration proof:** exact external adapter factory registration and event propagation using current pinned crates.  
**No permitted fallback to Python or upstream Core patches.** If a required API is unavailable, search public Rust composition alternatives; otherwise flag the feature as blocked rather than silently changing constraints.
