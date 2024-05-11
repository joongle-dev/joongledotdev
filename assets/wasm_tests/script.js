import init, { TestStruct } from './wasm_tests.js';

await init();

let struct = new TestStruct();

setInterval(function () { console.log(struct.count() )}, 1);

console.log(struct.count());