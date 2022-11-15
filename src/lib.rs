use anyhow::Result;
use cap_std::fs::Dir;
use log::info;
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

pub struct Limits {
    resources: u64,
}

impl Limits {
    fn instances(n: u64) -> Self {
        Self { resources: n }
    }
}

impl ResourceLimiter for Limits {
    fn instances(&self) -> usize {
        self.resources as usize
    }

    fn memories(&self) -> usize {
        self.resources as usize
    }

    fn tables(&self) -> usize {
        self.resources as usize
    }

    fn memory_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> bool {
        true
    }

    fn table_growing(&mut self, _current: u32, _desired: u32, _maximum: Option<u32>) -> bool {
        true
    }
}

pub struct Context {
    pub wasi: WasiCtx,
    pub limits: Limits,
}

impl Default for Context {
    fn default() -> Self {
        let builder = WasiCtxBuilder::new();
        let dir = Dir::open_ambient_dir(".", cap_std::ambient_authority()).unwrap();
        let wasi = builder
            // Allow access to the cwd, to read benchmark inputs
            .preopened_dir(dir, ".")
            .unwrap()
            .build();

        Self {
            wasi,
            limits: Limits::instances(100000),
        }
    }
}

#[derive(Default)]
pub struct VMOptions {
    pub fuel: bool,
    pub epoch_interruption: bool,
}

pub struct VM {
    linker: Linker<Context>,
    pub opts: VMOptions,
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl VM {
    pub fn new(opts: VMOptions) -> Self {
        let triple = target_lexicon::Triple::host();
        // Default to wasmtime's cli options
        let options = wasmtime_cli_flags::CommonOptions::parse_from_str("").unwrap();
        let mut config = options.config(Some(&triple.to_string())).unwrap();
        config.consume_fuel(opts.fuel);
        config.epoch_interruption(opts.epoch_interruption);

        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::sync::add_to_linker(&mut linker, |ctx: &mut Context| &mut ctx.wasi).unwrap();

        // Stub out imports expected by the Wasm modules
        // defined by sightglass
        linker.func_wrap("bench", "start", || {}).unwrap();
        linker.func_wrap("bench", "end", || {}).unwrap();

        Self { linker, opts }
    }

    pub fn with_fuel() -> Self {
        let mut opts = VMOptions::default();
        opts.fuel = true;
        Self::new(opts)
    }

    pub fn compile(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        self.linker.engine().precompile_module(bytecode)
    }

    pub fn deserialize(&self, bytes: &[u8]) -> Result<Module> {
        unsafe { Module::deserialize(&self.linker.engine(), bytes) }
    }

    pub fn exec(&self, store: &mut Store<Context>, module: &Module) -> Result<()> {
        let instance = self.linker.instantiate(store.as_context_mut(), module)?;
        let start = instance.get_typed_func::<(), (), _>(store.as_context_mut(), "_start")?;

        if self.opts.fuel {
            store.add_fuel(u64::MAX)?;
        }

        if self.opts.epoch_interruption {
            // 5 ms
            store.set_epoch_deadline(5u64);
            store.epoch_deadline_trap();
            let engine = store.engine().clone();

            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(1));
                engine.increment_epoch();
            });
        }

        let res = start.call(store.as_context_mut(), ());
        if res.is_err() {
            info!("Interrupted");
        }

        Ok(())
    }

    pub fn make_store(&self) -> Store<Context> {
        let context = Context::default();
        let store = Store::new(&self.linker.engine(), context);
        // store.limiter(|s| &mut s.limits);
        store
    }
}
