use criterion::{criterion_group, criterion_main, Criterion};

const BENCH_ROOT: &'static str = "sightglass/benchmarks-next/";

macro_rules! def_compile_bench {
    ($func:ident, $name:expr, $with_fuel:ident) => {
        fn $func(c: &mut Criterion) {
            let name = $name.replace("_", "-");
            let root = std::path::Path::new(BENCH_ROOT).join(&name);
            let input = std::fs::read(root.join("benchmark.wasm")).unwrap();
            let input_size = input.len();

            let with_fuel_id = stringify!($with_fuel);
            let with_fuel_id = with_fuel_id.replace("_", "-");
            let id = criterion::BenchmarkId::new(
                &format!("compile:{}:{}", &name, &with_fuel_id),
                format!("{} bytes", input_size),
            );
            let vm = wasm_bench::VM::$with_fuel();

            c.bench_with_input(id, &input, |b, input| {
                b.iter(|| vm.compile(input).unwrap());
            });
        }
    };
}

macro_rules! def_exec_bench {
    ($func:ident, $name:expr, $with_fuel:ident) => {
        fn $func(c: &mut Criterion) {
            let cwd = std::env::current_dir().unwrap();
            let name = $name.replace("_", "-");
            let root = std::path::Path::new(BENCH_ROOT).join(&name);
            let with_fuel_id = stringify!($with_fuel);
            let with_fuel_id = with_fuel_id.replace("_", "-");

            assert!(std::env::set_current_dir(&root).is_ok());
            let input = std::fs::read("benchmark.wasm").unwrap();
            let input_size = input.len();
            let id = criterion::BenchmarkId::new(
                &format!("exec:{}:{}", &name, &with_fuel_id),
                format!("{} bytes", input_size),
            );
            let vm = wasm_bench::VM::$with_fuel();
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

/// An iterator macro that defines the benchmarking matrix.
/// Benchmark entries should be of the form:
/// @<compile | exec> <default | with_fuel> <benchmark function definition> => <benchmark_directory>
macro_rules! for_each_bench {
    ($mac:ident) => {
        $mac! {
            @compile default compile_pulldown_cmark => pulldown_cmark
            @compile with_fuel compile_pulldown_cmark_with_fuel => pulldown_cmark
            @compile default compile_bz2 => bz2
            @compile with_fuel compile_bz2_with_fuel => bz2
            @compile default compile_shootout_base64 => shootout_base64
            @compile with_fuel compile_shootout_base64_with_fuel => shootout_base64
            @compile default compile_shootout_fib2 => shootout_fib2
            @compile with_fuel compile_shootout_fib2_with_fuel => shootout_fib2
            @compile default compile_shootout_heapsort => shootout_heapsort
            @compile with_fuel compile_shootout_heapsort_with_fuel  => shootout_heapsort
            @compile default compile_shootout_keccak => shootout_keccak
            @compile with_fuel compile_shootout_keccak_with_fuel => shootout_keccak
            @compile default compile_shootout_ed25519 => shootout_ed25519
            @compile with_fuel compile_shootout_ed25519_with_fuel => shootout_ed25519

            @exec default exec_shootout_ed25519 => shootout_ed25519
            @exec with_fuel exec_shootout_ed25519_with_fuel => shootout_ed25519
            @exec default exec_pulldown_cmark => pulldown_cmark
            @exec with_fuel exec_pulldown_cmark_with_fuel => pulldown_cmark
            @exec default exec_shootout_keccak => shootout_keccak
            @exec with_fuel exec_shootout_keccak_with_fuel => shootout_keccak
            // @exec default exec_shootout_heapsort => shootout_heapsort
            // @exec with_fuel exec_shootout_heapsort_with_fuel => shootout_heapsort
            @exec default exec_shootout_fib2 => shootout_fib2
            @exec with_fuel exec_shootout_fib2_with_fuel => shootout_fib2
            @exec default exec_shootout_base64 => shootout_base64
            @exec with_fuel exec_shootout_base64_with_fuel => shootout_base64
            // @exec default exec_bz2 => bz2
            // @exec with_fuel exec_bz2_with_fuel => bz2
        }
    };
}

/// Macro to define either a compilation or execution benchmark.
///
/// Benchmarks defined through this macro, will follow a specific
/// naming convention;
/// * Compilation benchmarks with fuel: `cargo run -- bench --name=compile:<name>:with_fuel`
/// * Compilation benchmarks without fuel: `cargo run -- bench --name=compile:<name>:default`
/// * Execution benchmarks with fuel: `cargo run -- bench --name=exec:<name>:with_fuel`
/// * Compilation benchmarks without fuel: `cargo run -- bench --name=exec:<name>:default`
macro_rules! def_bench {
    ( @compile $ty:ident $def:ident => $name:ident $($rest:tt)* ) => {
	def_compile_bench!($def, stringify!($name), $ty);
	def_bench!($($rest)*);
    };

    ( @exec $ty:ident $def:ident => $name:ident $($rest:tt)* ) => {
	def_exec_bench!($def, stringify!($name), $ty);
	def_bench!($($rest)*);
    };

    () => {};
}

for_each_bench!(def_bench);

criterion_group!(
    benches,
    compile_pulldown_cmark,
    compile_pulldown_cmark_with_fuel,
    exec_pulldown_cmark,
    exec_pulldown_cmark_with_fuel,
    compile_bz2,
    compile_bz2_with_fuel,
    // There's a weird error coming from Criterion's reporting infraestructure here.
    // An error related to too many files
    // exec_bz2,
    compile_shootout_base64,
    compile_shootout_base64_with_fuel,
    exec_shootout_base64,
    exec_shootout_base64_with_fuel,
    compile_shootout_fib2,
    compile_shootout_fib2_with_fuel,
    exec_shootout_fib2,
    exec_shootout_fib2_with_fuel,
    compile_shootout_heapsort,
    compile_shootout_heapsort_with_fuel,
    // Exits with exit code 1
    // exec_shootout_heapsort
    compile_shootout_keccak,
    compile_shootout_keccak_with_fuel,
    exec_shootout_keccak,
    exec_shootout_keccak_with_fuel,
    compile_shootout_ed25519,
    compile_shootout_ed25519_with_fuel,
    exec_shootout_ed25519,
    exec_shootout_ed25519_with_fuel
);

criterion_main!(benches);
