# Official Adapter Integration Contract — nautilus_ctp

Date: 2026-11-11  
Sources (official upstream):
- [Rust-native Adapters guide](https://nautilustrader.io/docs/latest/developer_guide/adapters/)
- [Python Adapter Interface (out-of-tree)](https://nautilustrader.io/docs/latest/developer_guide/python_adapters/)
- [Plugins (not an established runtime loading path)](https://nautilustrader.io/docs/latest/developer_guide/plugins/)

**Project hard constraints:** implement and document all CTP behavior in standalone `ligl/nautilus_ctp`. Never modify/fork/patch NautilusTrader Core. Verify behavior using an ordinary installed NautilusTrader release.

## 1. Crucial distinction: in-tree vs out-of-tree

The official Rust-native adapter guide explicitly covers **in-tree** Rust adapters. Its steps for adding a Rust workspace member, PyO3 registry and `nautilus_trader._libnautilus` module require modifying the upstream Nautilus source tree: **these steps are prohibited for this project**.

The official **Python Adapter Interface** explicitly supports custom adapters in a separate Python package, optionally backed by an independently compiled Rust/PyO3 extension. Its documented integration boundary is the Python client interface and wheel-owned Nautilus domain classes, **not an arbitrary cross-module Rust trait-object ABI**. Therefore:
- Prefer: **Rust CTP SDK wrapper / protocol + Rust/PyO3 extension + thin Python DataClient / ExecutionClient façade**.
- Register with the official Python client factories using `LiveNodeBuilder.add_data_client` / `add_exec_client`, or documented config-based factories.
- Do not claim a standalone Rust crate can be loaded directly as a Rust-native in-tree `DataClient`/`ExecutionClient` without further evidence.
- The early-alpha `nautilus-plugin` ABI is **not** a substitute: upstream says the crate only defines artifact identity/boundary and does not load, register or run plugins; its host is internal.
- All Rust-owned CTP logic, including position ledger, sessions, order identity, normalization and session-aware bars, stays in this independent repo; the Python façade should only adapt lifecycle, commands and typed events.
- Investigate direct external Rust registration only if an officially supported public host interface is later demonstrated and tested. It is not the baseline.

## 2. Consequences for existing design documents

The proposed external Rust `CtpExecutionClient: nautilus_common::clients::ExecutionClient` is a **conceptual contract only** for out-of-tree delivery, not a verified plug-in integration technique. Replace deployment assumptions with the supported Python client registration and Rust backend delegation. Preserve Rust as the source of truth for all protocol and CTP domain logic. The standalone repository can remain independent.

Be careful with a documented limitation: custom Python clients currently do not support Redis/PostgreSQL cache backing in either launch mode. This affects durable reconciliation and must be tracked as a P0 deployment/operational restriction, especially if restart recovery depends on core cache persistence. The adapter must durably persist its own CTP order identity/ledger, but that alone does not prove the complete node can recover all Nautilus execution state. Verify a supported deployment mode end-to-end before a production claim.

## 3. Official lifecycle and state ownership

- Factory construction does not open SDK/front connections; initialize in `_connect` only after runtime/event-loop binding.
- Read-only `ClientCache` inspection is allowed on owner thread; never mutate Nautilus Core Cache or use it from CTP SDK callback thread.
- Emit `QuoteTick`, `Bar`, `Instrument`, `OrderStatusReport`, `FillReport`, `PositionStatusReport` and execution events via documented typed client methods, constructing values using **model classes from the installed Nautilus wheel**.
- CTP callbacks must cross an owned, thread-safe boundary into the Python client's owner loop; do not create/manipulate wheel-owned PyObjects or Core Cache on arbitrary CTP SPI threads.
- `_connect` must not claim readiness until instrument/account bootstrap and command paths are valid; execution reconciliation precedes strategy trading.
- `_disconnect` closes resources and joins Rust workers; commands and subscription intent must have explicit cancellation/recovery semantics.
- Factory + Python façade are registered via documented public APIs, not upstream registry modifications.

## 4. CTP session and bar specifics

- Market TradingSessions and CTP FrontID/SessionID network sessions remain **separate** domains.
- Use public typed `Bar` output for CTP-aware external bars when INTERNAL aggregation cannot honor China futures breaks/holidays; test explicit boundaries, no-update handling, cumulative volume reset and backtest/live equivalence.
- The Python interface offers `_subscribe_bars`, `_request_bars` hooks. Full bar subscription lists and subscription ownership are part of public behavior.
- No faux `TradeTick` semantics: a CTP depth-market snapshot with inferred volume change is not automatically a venue-authenticated individual trade.
- Reconcile unknown orders through authoritative evidence and order/fill/position reports; do not blindly resubmit.

## 5. Official test / release gates

- **Installed-wheel isolation**: build an independent adapter wheel, install alongside a stock non-editable NautilusTrader wheel in a fresh virtual environment. Check import identity to prevent duplicate Nautilus PyO3 classes.
- **Startup**: both `node.run()` and hosted `run_async()` modes when supported; verify account bootstrap and reconciliation complete.
- **Data**: one CTP-derived quote and one CTP session-aware Bar reaches strategy and core cache.
- **Execution**: deterministic mock order, fill price/quantity/commission, cancel, unknown-outcome recovery and final cached state.
- **Shutdown**: no leaked CTP worker, late event or cache/output access after dispose.
- **Deployment persistence**: explicitly test live restart with and without core cache database; currently advertised unsupported backing for Python clients prevents production approval of any mode relying on it.
- **Nautilus Data / Execution Testing Specs**: apply to every claimed feature and record unsupported test cases by venue/protocol reason.

## 6. Revised assessment

**Official out-of-tree integration path FOUND:** independent Rust/PyO3 package + Python adapter façade.  
**Direct out-of-tree Rust trait registration:** NOT ESTABLISHED by official guide.  
**Full CTP compatibility:** NOT YET PROVEN.  
**Material additional deployment gap:** Python-client cache persistence restriction.  
**Core modifications:** PROHIBITED.

This supersedes any earlier statements that out-of-tree Rust trait registration is already verified or that an in-tree factory registration plan is valid here.
