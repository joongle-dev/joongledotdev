#!/bin/bash
export RUSTFLAGS=--cfg=web_sys_unstable_apis
cargo build --target=wasm32-unknown-unknown --release
wasm-bindgen --web --no-typescript --out-dir=../assets/yahtzee target/wasm32-unknown-unknown/release/yahtzee.wasm
wasm-opt -o ../assets/yahtzee/yahtzee_bg.wasm -Os ../assets/yahtzee/yahtzee_bg.wasm
gzip -k ../assets/yahtzee/yahtzee_bg.wasm
read pause