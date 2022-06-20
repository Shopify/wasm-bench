// V8
// * With liftoff only: node --experimental-wasi-unstable-preview1 --liftoff --no-wasm-tier-up index.mjs
// * With turbofan only: node --experimental-wasi-unstable-preview1 --no-liftoff --no-wasm-tier-up index.mjs
//
import * as fs from 'fs';
import * as path from 'path';
import { WASI } from 'wasi';


fs.open('log', 'w+', function(e, fd) {
  const wasi = new WASI({
    preopens: {
      '.': '.'
    },
    // Small hack avoid inheriting stdout/stderr
    stdout: fd,
    stderr:  fd,
  });
  const imports = {
    wasi_snapshot_preview1: wasi.wasiImport,
    bench: {
      start: function() {},
      end: function() {},
    }
  };

  var benchPath = process.argv[2];
  var bytecode = fs.readFileSync(benchPath);

  var m = new WebAssembly.Module(bytecode);
  var instance = new WebAssembly.Instance(m, imports);

  var start = performance.now();
  wasi.start(instance);
  var tt = performance.now() - start;

  console.info(`Execution took: ${tt}ms`);
});

