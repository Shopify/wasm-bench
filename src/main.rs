const BENCH_ROOT: &'static str = "sightglass/benchmarks-next/";

use wasm_bench::VM;

fn main() {
    let p = precision::Precision::new(precision::Config::default()).unwrap();
    let  bytecode = std::fs::read(std::path::Path::new(BENCH_ROOT).join("pulldown-cmark").join("benchmark.wasm")).unwrap();

    let vm: VM = Default::default();

    let start = p.now();
    let code = vm.compile(&bytecode).unwrap();
    let end = p.now() - start;

    println!("Code length: {} bytes", code.len());
    println!("{} ms", end.as_millis(&p));
}
