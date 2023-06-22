use std::fmt::{Debug, Formatter};
use wasmtime::{Config, Engine, Instance, Module, Store};


#[allow(dead_code)]
pub struct WasmRunner {
    store: Store<ContextView>,
    instance: Instance,
}

impl Debug for WasmRunner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("WasiRunner")
    }
}

impl WasmRunner {
    pub fn run_wasm(wasm: &[u8]) -> anyhow::Result<Self> {
        let engine = get_engine()?;
        let module = Module::new(&engine, wasm)?;
        let mut store = Store::new(&engine, ContextView {});
        let instance = Instance::new(&mut store, &module, &[])?;
        Ok(Self {
            store,
            instance,
        })
    }
}

fn get_engine() -> anyhow::Result<Engine> {
    let mut config = Config::new();
    {
        // config.async_support(false);
        // config.wasm_component_model(false);
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

pub struct ContextView {}