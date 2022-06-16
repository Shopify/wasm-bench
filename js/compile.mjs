// V8
// * With liftoff only: node --liftoff --no-wasm-tier-up index.mjs
// * With turbofan only: node --no-liftoff --no-wasm-tier-up index.mjs
//
import * as fs from 'fs';

var path = process.argv[2];
var bytecode = fs.readFileSync(path);


var start = performance.now();
var m = new WebAssembly.Module(bytecode);
console.log(m);
var tt = performance.now() - start;

console.log(`${tt}ms`);


