#!/bin/bash
export RUSTFLAGS=--cfg=web_sys_unstable_apis
cargo build --target=wasm32-unknown-unknown --release
wasm-bindgen --web --out-dir=../assets/yahtzee target/wasm32-unknown-unknown/release/yahtzee.wasm