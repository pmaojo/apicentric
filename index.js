// JavaScript wrapper for the generated WebAssembly module
const wasm = require('./pkg/mockforge.js');

function greet(name) {
  return wasm.greet(name);
}

module.exports = {
  greet
};

