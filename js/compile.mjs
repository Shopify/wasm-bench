// V8
// * With liftoff only: node --liftoff --no-wasm-tier-up index.mjs
// * With turbofan only: node --no-liftoff --no-wasm-tier-up index.mjs
//
import * as fs from 'fs';
import bench from 'nanobench';

bench('[Liftoff] Compilation', function(b) {
  var path = process.argv[2];
  var bytecode = fs.readFileSync(path);

  b.start();
  var m = new WebAssembly.Module(bytecode);
  b.end();
  console.log(m);
});


