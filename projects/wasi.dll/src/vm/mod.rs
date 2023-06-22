use std::fmt::{Debug, Formatter};
use wasmtime::{
    component::{Component, Instance, Linker, ResourceTable},
    Config, Engine, Store,
};
use wasmtime_wasi::preview2::{WasiCtx, WasiCtxBuilder, WasiView};

/// The wasi running environment
#[allow(dead_code)]
pub struct WasiRunner {
    store: Store<ContextView>,
    instance: Instance,
}

impl Debug for WasiRunner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("WasiRunner")
    }
}

impl WasiRunner {
    /// Run a wasm bytecodes
    pub fn run_wasm(bytecode: &[u8]) -> anyhow::Result<Self> {
        let engine = get_engine()?;
        let component = Component::from_binary(&engine, bytecode)?;
        get_component(engine, component)
    }
}

fn get_engine() -> anyhow::Result<Engine> {
    let mut config = Config::new();
    {
        config.async_support(false);
        config.wasm_component_model(true);
    }
    {
        config.debug_info(true);
        config.wasm_backtrace(true);
    }
    {
        // config.wasm_gc(true);
        config.wasm_function_references(true);
        config.wasm_reference_types(true);
        config.wasm_memory64(true);
    }
    Engine::new(&config)
}

fn get_component(engine: Engine, input: Component) -> anyhow::Result<WasiRunner> {
    let mut store = {
        let mut builder = WasiCtxBuilder::new();
        builder.inherit_stderr();
        builder.inherit_stdout();
        builder.inherit_stdin();
        Store::new(&engine, ContextView::new(ResourceTable::default(), builder.build()))
    };
    let instance = {
        let mut linker = Linker::<ContextView>::new(&engine);
        linker.allow_shadowing(true);
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)?;
        linker.instantiate(&mut store, &input)?
    };
    Ok(WasiRunner { store, instance })
}

pub struct ContextView {
    wasi: WasiCtx,
    resources: ResourceTable,
}

impl ContextView {
    fn new(table: ResourceTable, wasi: WasiCtx) -> Self {
        Self { resources: table, wasi }
    }
}

impl WasiView for ContextView {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
