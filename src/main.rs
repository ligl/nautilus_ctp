//! Out-of-tree, pure-Rust Nautilus client factory registration smoke test.
//! Source-reviewed against upstream 4fe0e3c3; NOT YET cargo-checked or run.
//! It tests registration only, NOT actual client construction or event delivery.
use std::{any::Any, cell::RefCell, rc::Rc};

use nautilus_common::{
    cache::CacheView,
    clients::{DataClient, ExecutionClient},
    clock::Clock,
    enums::Environment,
    factories::{ClientConfig, DataClientFactory, ExecutionClientFactory},
};
use nautilus_live::node::LiveNode;
use nautilus_model::identifiers::{TraderId};

#[derive(Debug)]
struct CtpMockConfig;

impl ClientConfig for CtpMockConfig {
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug)]
struct CtpMockDataFactory;

impl DataClientFactory for CtpMockDataFactory {
    fn create(
        &self,
        _name: &str,
        _config: &dyn ClientConfig,
        _cache: CacheView,
        _clock: Rc<RefCell<dyn Clock>>,
    ) -> anyhow::Result<Box<dyn DataClient>> {
        anyhow::bail!("CTP mock data factory intentionally has no client")
    }

    fn name(&self) -> &str { "CTP-MOCK-DATA" }
    fn config_type(&self) -> &str { "CtpMockConfig" }
}

#[derive(Debug)]
struct CtpMockExecFactory;

impl ExecutionClientFactory for CtpMockExecFactory {
    fn create(
        &self,
        _trader_id: TraderId,
        _name: &str,
        _config: &dyn ClientConfig,
        _cache: CacheView,
        _clock: Rc<RefCell<dyn Clock>>,
    ) -> anyhow::Result<Box<dyn ExecutionClient>> {
        anyhow::bail!("CTP mock execution factory intentionally has no client")
    }

    fn name(&self) -> &str { "CTP-MOCK-EXEC" }
    fn config_type(&self) -> &str { "CtpMockConfig" }
}

fn main() -> anyhow::Result<()> {
    // This path is intentionally independent of the Nautilus source workspace.
    let _builder = LiveNode::builder(
        TraderId::from("CTP-001"),
        Environment::Sandbox,
    )?
    .add_data_client(None, Box::new(CtpMockDataFactory), Box::new(CtpMockConfig))?
    .add_exec_client(None, Box::new(CtpMockExecFactory), Box::new(CtpMockConfig))?;

    println!("External Rust data/execution factories accepted by LiveNodeBuilder");
    Ok(())
}
