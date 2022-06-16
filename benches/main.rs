use criterion::{criterion_group, criterion_main, Criterion};

const BENCH_ROOT: &'static str = "sightglass/benchmarks-next/";

macro_rules! def_compile_bench {
    ($func:ident, $name:expr) => {
        fn $func(c: &mut Criterion) {
            let root = std::path::Path::new(BENCH_ROOT).join($name);
            let input = std::fs::read(root.join("benchmark.wasm")).unwrap();
            let input_size = input.len();
            let id = criterion::BenchmarkId::new(&format!("compile:{}", $name), format!("{} bytes", input_size));
            let vm = wasm_bench::VM::default();

            c.bench_with_input(id, &input, |b, input| {
                b.iter(|| {
                    vm.compile(input).unwrap()
                });
            });
        }
    };
}


macro_rules! def_exec_bench {
    ($func:ident, $name:expr) => {
        fn $func(c: &mut Criterion) {
            let cwd = std::env::current_dir().unwrap();
            let root = std::path::Path::new(BENCH_ROOT).join($name);

            assert!(std::env::set_current_dir(&root).is_ok());
            let input = std::fs::read("benchmark.wasm").unwrap();
            let input_size = input.len();
            let id = criterion::BenchmarkId::new(&format!("execute:{}", $name), format!("{} bytes", input_size));
            let vm = wasm_bench::VM::default();
            let mut store = vm.make_store();
            let bytes = vm.compile(&input).unwrap();
            let module = vm.deserialize(&bytes).unwrap();

            c.bench_with_input(id, &module, |b, module| {
                b.iter(|| {
                    vm.exec(&mut store, &module).unwrap();
                });
            });
            assert!(std::env::set_current_dir(&cwd).is_ok());
        }
    };
}


def_compile_bench!(compile_pulldown_cmark, "pulldown-cmark");
def_exec_bench!(exec_pulldown_cmark, "pulldown-cmark");

def_compile_bench!(compile_bz2, "bz2");
def_exec_bench!(exec_bz2, "bz2");

def_compile_bench!(compile_shootout_base64, "shootout-base64");
def_exec_bench!(exec_shootout_base64, "shootout-base64");

def_compile_bench!(compile_shootout_fib2, "shootout-fib2");
def_exec_bench!(exec_shootout_fib2, "shootout-fib2");

def_compile_bench!(compile_shootout_heapsort, "shootout-heapsort");
def_exec_bench!(exec_shootout_heapsort, "shootout-heapsort");

def_compile_bench!(compile_shootout_keccak, "shootout-keccak");
def_exec_bench!(exec_shootout_keccak, "shootout-keccak");

def_compile_bench!(compile_shootout_ed25519, "shootout-ed25519");
def_exec_bench!(exec_shootout_ed25519, "shootout-ed25519");

criterion_group!(
    benches,
    compile_pulldown_cmark,
    exec_pulldown_cmark,
    compile_bz2,
    // There's a weird error coming from Criterion's reporting infraestructure here.
    // An error related to too many files
    // exec_bz2,
    compile_shootout_base64,
    exec_shootout_base64,
    compile_shootout_fib2,
    exec_shootout_fib2,
    compile_shootout_heapsort,
    // Exits with exit code 1
    // exec_shootout_heapsort
    compile_shootout_keccak,
    exec_shootout_keccak,
    compile_shootout_ed25519,
    exec_shootout_ed25519
);
criterion_main!(benches);
