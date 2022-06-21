// SpiderMonkey
// Run with: WASM_PATH=path/to/binary path/to/mach run -- --wasm-compiler=<optimizing|baseline> js/index.js  
// If cranelift is enabled, optimizing -> cranelift, else optimizing -> ion;
// this depends on your mozconfig settings
//
// For reference (mozconfig)
// 
// ac_add_options --enable-project=js
// ac_add_options --enable-application=js
// ac_add_options --enable-optimize
// ac_add_options --disable-debug
// ac_add_options --enable-cranelift
// ac_add_options --enable-spidermonkey-telemetry
// 
var ITERATIONS = 30;
var elapsed = 0;
var path = os.getenv("WASM_PATH");
var bytecode = os.file.readFile(path, 'binary');

for (var i = 0; i < ITERATIONS; i++) {
  var start = performance.now();
  var m = new WebAssembly.Module(bytecode);
  console.log(m);
  var end = performance.now() - start;
  elapsed += end;
}


console.log(`
  Compiler: ${wasmCompileMode()},
  Compilation took: ${elapsed / ITERATIONS}ms
`);

