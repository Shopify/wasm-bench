use anyhow::{Context, Result};
use log::info;
use std::process::Command;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use precision::{Config, Precision};
use structopt::StructOpt;
use wasm_bench::{VMOptions, VM};

const BENCH_ROOT: &'static str = "sightglass/benchmarks-next/";

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    commands: Commands,
}

#[derive(StructOpt)]
enum Commands {
    SetupGecko(SetupGeckoOpts),
    Bench(BenchOpts),
}

#[derive(StructOpt)]
struct SetupGeckoOpts {
    #[structopt(long, default_value = "release")]
    _profile: String,
}

#[derive(StructOpt)]
struct BenchOpts {
    #[structopt(long)]
    name: String,
    #[structopt(long)]
    time_stats: bool,
    #[structopt(long)]
    fuel: bool,
    #[structopt(long)]
    epoch_interruption: bool,
    #[structopt(long)]
    js: bool,
    #[structopt(long)]
    single: bool,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let precision = Precision::new(Config::default()).unwrap();

    match opt.commands {
        Commands::SetupGecko { .. } => {
            setup_gecko().unwrap();
        }
        Commands::Bench(opts) => {
            if opts.single {
                let mut vm_opts = VMOptions::default();
                vm_opts.fuel = opts.fuel;
                vm_opts.epoch_interruption = opts.epoch_interruption;
                let vm = VM::new(vm_opts);
                let bytes = compile(&vm, &opts.name, &precision).unwrap();
                info!("Machine code size: {} bytes", bytes.len());
                exec(&vm, &opts.name, &precision).unwrap();
            }

            if opts.time_stats {
                info!("[Cranelift] Starting benchmark");
                cmd(&["cargo", "bench", "--", &opts.name], None, None);
            }

            if opts.js {
                compile_with_liftoff(&opts.name).unwrap();
                compile_with_rabaldr(&opts.name).unwrap();
                exec_with_liftoff(&opts.name).unwrap();
            }
        }
    }
}

fn cmd(
    command: &[&str],
    working_directory: Option<PathBuf>,
    env: Option<&HashMap<String, String>>,
) {
    info!("> {}", command.join(" "));
    let mut cmd = Command::new(command[0]);
    cmd.args(&command[1..]);
    if let Some(working_directory) = working_directory {
        cmd.current_dir(working_directory);
    }

    if let Some(env) = env {
        for (k, v) in env.into_iter() {
            cmd.env(k, v);
        }
    }

    cmd.status().expect("unable to execute command");
}

fn compile(vm: &VM, name: &str, precision: &Precision) -> Result<Vec<u8>> {
    info!("====== [Cranelift] Starting compilation ======");

    let bytecode = fs::read(path_from_name(name).join("benchmark.wasm"))
        .with_context(|| format!("Benchmark not found: {}", name))?;

    let start = precision.now();
    let code = vm.compile(&bytecode).unwrap();
    let end = precision.now() - start;

    info!("Binary size: {} bytes", bytecode.len());
    info!("Compilation took: {}ms", end.as_millis(precision));

    Ok(code)
}

fn path_from_name(name: &str) -> PathBuf {
    Path::new(BENCH_ROOT).join(name)
}

fn exec(vm: &VM, name: &str, precision: &Precision) -> Result<()> {
    info!("====== [Cranelift] Starting execution ======");
    let cwd = env::current_dir()?;
    let root = path_from_name(name);

    assert!(env::set_current_dir(root).is_ok());
    let bytecode = fs::read("benchmark.wasm")?;
    let bytes = vm.compile(&bytecode)?;
    let module = vm.deserialize(&bytes)?;
    let mut store = vm.make_store();

    let start = precision.now();
    vm.exec(&mut store, &module)?;
    let end = precision.now() - start;
    info!("Execution took: {} ms", end.as_millis(precision));
    if let Some(consumed) = store.fuel_consumed() {
        info!("Execution consumed {} fuel", consumed);
    }
    assert!(env::set_current_dir(cwd).is_ok());

    Ok(())
}

fn exec_with_liftoff(name: &str) -> Result<()> {
    info!("====== [Liftoff] Starting execution ======");
    let cwd = env::current_dir()?;
    let bench_path = path_from_name(name);
    let js_path = cwd.join("js").join("execute.mjs");
    let js_path = js_path.to_str().context("Could not convert to &str")?;
    assert!(env::set_current_dir(bench_path).is_ok());
    cmd(
        &[
            "node",
            "--experimental-wasi-unstable-preview1",
            "--no-wasm-tier-up",
            "--liftoff",
            js_path,
            "benchmark.wasm",
        ],
        None,
        None,
    );
    assert!(env::set_current_dir(cwd).is_ok());

    Ok(())
}

fn compile_with_liftoff(name: &str) -> Result<()> {
    info!("====== [Liftoff] Starting compilation ======");
    let path = path_from_name(name).join("benchmark.wasm");
    let path = path
        .as_os_str()
        .to_str()
        .context("Couldn't convert to str")?;

    cmd(
        &[
            "node",
            "--liftoff",
            "--no-wasm-tier-up",
            "js/compile.mjs",
            &path,
        ],
        None,
        None,
    );

    Ok(())
}

fn compile_with_rabaldr(name: &str) -> Result<()> {
    info!("====== [RabaldrMonkey] Starting compilation ======");
    let cwd = env::current_dir()?;
    let gecko_path = cwd.join("gecko-dev");
    let source_path = cwd.join("js").join("compile.js");
    let source_path = source_path
        .as_os_str()
        .to_str()
        .context("Could not convert to &str")?;
    let path = cwd.join(path_from_name(name).join("benchmark.wasm"));
    let path = path
        .as_os_str()
        .to_str()
        .context("Couldn't convert to &str")?;

    let mut env = HashMap::new();
    env.insert("WASM_PATH".into(), path.into());

    cmd(
        &[
            "./mach",
            "run",
            "--",
            "--wasm-compiler=baseline",
            &source_path,
        ],
        Some(gecko_path),
        Some(&env),
    );

    Ok(())
}

fn setup_gecko() -> Result<()> {
    let cwd = env::current_dir()?;
    let moz_config_path = cwd.join("mozconfigs").join("release");
    let moz_config_path = moz_config_path
        .to_str()
        .context("Could not convert to String")?;
    let mut env = HashMap::new();
    env.insert("MOZCONFIG".into(), moz_config_path.into());

    cmd(
        &["./mach", "build"],
        Some(cwd.join("gecko-dev")),
        Some(&env),
    );

    Ok(())
}
