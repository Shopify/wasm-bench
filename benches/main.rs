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
 
fn compilation(c: &mut Criterion) {
    let vm: VM = Default::default();

    def_compile_bench!(c, "pulldown-cmark", vm);
}

criterion_group!(benches, compilation);
criterion_main!(benches);
