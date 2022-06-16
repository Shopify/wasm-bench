use wasmtime::*;
use anyhow::Result;

pub struct VM {
    engine: Engine,
}

impl Default for VM {
    fn default() -> Self {
        let triple = target_lexicon::Triple::host();
        // Default to wasmtime's cli options
        let options = wasmtime_cli_flags::CommonOptions::parse_from_str("").unwrap();
        let config = options.config(Some(&triple.to_string())).unwrap();
        let engine = Engine::new(&config).unwrap();
        Self { engine }
    }
}

impl VM {
    pub fn compile(&self, bytecode: &[u8]) -> Result<Vec<u8>> {
        self.engine.precompile_module(bytecode)
    }
}
