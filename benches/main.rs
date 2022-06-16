use criterion::{criterion_group, criterion_main, Criterion};
use wasm_bench::VM;

const BENCH_ROOT: &'static str = "sightglass/benchmarks-next/";

macro_rules! def_compile_bench {
    ($c:expr, $name:expr, $vm:expr) => {
        let root = std::path::Path::new(BENCH_ROOT).join($name);
        let input = std::fs::read(root.join("benchmark.wasm")).unwrap();
        let input_size = input.len();
        let id = criterion::BenchmarkId::new(&format!("compilation:{}", $name), format!("{} bytes", input_size));

        $c.bench_with_input(id, &input, |b, input| {
            b.iter(|| {
                $vm.compile(input).unwrap()
            });
        });
    };
}


macro_rules! def_exec_bench {
    ($c:expr, $name:expr, $vm:expr) => {
        let cwd = std::env::current_dir().unwrap();
        let root = std::path::Path::new(BENCH_ROOT).join($name);

        assert!(std::env::set_current_dir(&root).is_ok());
        let input = std::fs::read("benchmark.wasm").unwrap();
        let input_size = input.len();
        let id = criterion::BenchmarkId::new(&format!("execution:{}", $name), format!("{} bytes", input_size));
        let mut store = $vm.make_store();
        let bytes = $vm.compile(&input).unwrap();
        let module = $vm.deserialize(&bytes).unwrap();

        $c.bench_with_input(id, &module, |b, module| {
            b.iter(|| {
                $vm.exec(&mut store, &module).unwrap();
            });
        });
        assert!(std::env::set_current_dir(&cwd).is_ok());
    };
}
 
fn compilation(c: &mut Criterion) {
    let vm: VM = Default::default();

    def_compile_bench!(c, "pulldown-cmark", vm);
}

fn execution(c: &mut Criterion) {
    let vm: VM = Default::default();

    def_exec_bench!(c, "pulldown-cmark", vm);
    def_exec_bench!(c, "shootout-fib2", vm);
}

criterion_group!(benches, compilation, execution);
criterion_main!(benches);
