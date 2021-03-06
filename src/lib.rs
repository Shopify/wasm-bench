use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use anyhow::Result;
use cap_std::fs::Dir;


pub struct Context {
    pub wasi: WasiCtx,
}

impl Default for Context {
    fn default() -> Self {
        let builder = WasiCtxBuilder::new();
        let dir = Dir::open_ambient_dir(".", cap_std::ambient_authority()).unwrap();
        let wasi = builder
            // Allow access to the cwd, to read benchmark inputs
            .preopened_dir(dir, ".").unwrap()
            .build();

        Self {
            wasi,
        }
    }
}

pub struct VM {
    linker: Linker<Context>,
}

impl Default for VM {
    fn default() -> Self {
        let triple = target_lexicon::Triple::host();
        // Default to wasmtime's cli options
        let options = wasmtime_cli_flags::CommonOptions::parse_from_str("").unwrap();
        let config = options.config(Some(&triple.to_string())).unwrap();
        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::sync::add_to_linker(&mut linker, |ctx: &mut Context| &mut ctx.wasi).unwrap();
        
        // Stub out imports expected by the Wasm modules 
        // defined by sightglass
        linker.func_wrap("bench", "start", || {}).unwrap();
        linker.func_wrap("bench", "end", || {}).unwrap();
        
        Self { linker }
    }
}

impl VM {
    pub fn compile(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        self.linker.engine().precompile_module(bytecode)
    }

    pub fn deserialize(&self, bytes: &[u8]) -> Result<Module> {
        unsafe { 
            Module::deserialize(&self.linker.engine(), bytes)
        }
    }

    pub fn exec(&self, store: &mut Store<Context>, module: &Module) -> Result<()> {
        let instance = self.linker.instantiate(store.as_context_mut(), module)?;
        let start = instance.get_typed_func::<(), (), _>(store.as_context_mut(), "_start")?;

        start.call(store.as_context_mut(), ())?;

        Ok(())
    }

    pub fn make_store(&self) -> Store<Context> {
        let context = Context::default();
        Store::new(&self.linker.engine(), context)
    }
}
