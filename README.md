# WebAssembly Bench

A WebAssembly benchmarking suite, built on top of the [Sightglass suite](https://github.com/bytecodealliance/sightglass/tree/main/benchmarks-next).

This suite **is not a rigorous benchmark across WebAssembly engines**; the intention is to provide a simple approximation and a _rough_ comparison between compilation and execution times of various WebAssembly modules using different WebAssembly compilers (Baseline, Optimizing).

## Compilers

* Optimizing: Cranelift (Wasmtime)
* Baseline: Liftoff (V8)
* Baseline: RabaldrMonkey (SpiderMonkey)

## Analysis

These benchmarks exist, to answer the following:

* Approximate compilation time gains when using a baseline compiler
* Approximate execution time difference when using a baseline compiler 

## Misc

This suite doesn't use Sightglass's CLI to run the benchmarks. Instead it uses Criterion for Rust benchmarks and JavaScript's `Performance` API for JavaScript benchmarks.

Sightglass uses Wasmtime's Benchmark API which intentionally uses a Shuffling Allocator to ensure statistically meaningful results, but make it unsuitable for comparisons between different engines.
