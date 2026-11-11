# Stage 0: Pure Rust out-of-tree integration smoke

## Purpose

Prove **a standalone Cargo package** can refer to the published Rust traits and register external factories using public `LiveNodeBuilder` methods without editing NautilusTrader Core. No Python/PyO3 or CTP SDK is involved.

Source contract:
- [Client factory traits](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/common/src/factories/client.rs)
- [LiveNodeBuilder registration methods](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/crates/live/src/node/builder.rs)
- [Run live trading in Rust](https://github.com/nautechsystems/nautilus_trader/blob/4fe0e3c3b60f05b31f8682f24609d5914ddd4267/docs/how_to/run_rust_live_trading.md)

## Instructions

Requires Rust toolchain >= 1.99.1 (upstream documented MSRV) and normal Cargo git/network access:

```bash
cargo check --bin nautilus-ctp-compat-smoke
cargo run --bin nautilus-ctp-compat-smoke
```

Expected program output after successful compile/run:

```text
External Rust data/execution factories accepted by LiveNodeBuilder
```

**Current evidence:** source-level signature inspection only. **NOT COMPILED / NOT EXECUTED**. This assistant's execution environment has no `cargo` and cannot resolve `github.com`, so compilation cannot be claimed.

This smoke proves only factory registration; factory `create` intentionally returns an error. Calling `build()` or `run()` is out of scope and expected to fail until functional stub clients are added. Next gate is implementation of complete `DataClient` and `ExecutionClient` mocks with `LiveNode.build`, start/stop, events, cache and reconciliation.

## Fixed upstream revision

`4fe0e3c3b60f05b31f8682f24609d5914ddd4267`

All Nautilus crates use that exact revision. Do not mix git and crates.io Nautilus versions or add `python` features.

## Status

| Gate | Status |
|---|---|
| Public trait and builder signatures inspected | PASS (source review) |
| Standalone Cargo package prepared | READY |
| cargo check | NOT RUN |
| cargo run | NOT RUN |
| LiveNode build and lifecycle | NOT IMPLEMENTED |
| Data/execution event propagation | NOT IMPLEMENTED |
| Session-aware bars and CTP semantics | NOT IMPLEMENTED |

This is validation infrastructure, not a production adapter.
