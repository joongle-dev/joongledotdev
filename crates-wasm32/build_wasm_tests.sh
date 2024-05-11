#!/bin/bash
cargo build -p wasm_tests --target=wasm32-unknown-unknown --release
wasm-bindgen --web --no-typescript --out-dir=../assets/wasm_tests target/wasm32-unknown-unknown/release/wasm_tests.wasm
wasm-opt -o ../assets/wasm_tests/wasm_tests_bg.wasm -Os ../assets/wasm_tests/wasm_tests_bg.wasm
gzip -kf ../assets/wasm_tests/wasm_tests_bg.wasm