//! The built player, instantiated in wasmi and called through ABI v1 as the loader calls it.

use anyhow::{Context, ensure};
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, WasmParams};

/// The status of a call that completed.
pub const DONE: u32 = 0;

/// An instance of the player module, without any import.
pub struct WasmPlayer {
    store: Store<()>,
    instance: Instance,
    memory: Memory,
}

impl WasmPlayer {
    /// Compiles and validates `module`, checks that it imports nothing, and instantiates it.
    pub fn instantiate(module: &[u8]) -> anyhow::Result<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, module).context("compiling the module")?;
        let imports: Vec<_> = module
            .imports()
            .map(|import| format!("{}::{}", import.module(), import.name()))
            .collect();
        ensure!(imports.is_empty(), "the module imports {}", imports.join(", "));
        let mut store = Store::new(&engine, ());
        let instance = Linker::new(&engine)
            .instantiate_and_start(&mut store, &module)
            .context("instantiating the module")?;
        let memory = instance
            .get_memory(&store, "memory")
            .context("the module exports no `memory`")?;
        Ok(Self {
            store,
            instance,
            memory,
        })
    }

    /// Calls the export `name` with `params`: every ABI v1 export returns a `u32`.
    pub fn call<Params: WasmParams>(&mut self, name: &str, params: Params) -> anyhow::Result<u32> {
        let export = self
            .instance
            .get_typed_func::<Params, u32>(&self.store, name)
            .with_context(|| format!("finding the export `{name}`"))?;
        export
            .call(&mut self.store, params)
            .with_context(|| format!("calling `{name}`"))
    }

    /// `alloc`, a copy of `payload` into memory, then `load`: the loader's steps.
    pub fn load(&mut self, payload: &[u8]) -> anyhow::Result<()> {
        let len = u32::try_from(payload.len()).context("the payload passes 4 GiB")?;
        let pointer = self.call("alloc", (len,))?;
        ensure!(pointer != 0, "alloc({len}) returned 0");
        self.memory
            .write(&mut self.store, address(pointer)?, payload)
            .context("writing the payload into memory")?;
        let status = self.call("load", (pointer, len))?;
        ensure!(status == DONE, "load returned status {status}");
        Ok(())
    }

    /// The `len` bytes of memory at `pointer`.
    pub fn read(&self, pointer: u32, len: u32) -> anyhow::Result<Vec<u8>> {
        let mut bytes = vec![0; usize::try_from(len)?];
        self.memory
            .read(&self.store, address(pointer)?, &mut bytes)
            .with_context(|| format!("reading {len} bytes of memory at {pointer}"))?;
        Ok(bytes)
    }
}

fn address(pointer: u32) -> anyhow::Result<usize> {
    Ok(usize::try_from(pointer)?)
}
