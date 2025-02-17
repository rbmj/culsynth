#!/bin/bash
set -e
cd `dirname $0`
mkdir -p static_debug
cargo build -p culsynth_plugin --target wasm32-unknown-unknown --no-default-features --features "audioworklet"
wasm-bindgen --target web --no-typescript --out-dir static_debug/pkg target/wasm32-unknown-unknown/debug/culsynth_plugin.wasm
cargo build -p culsynth_web --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir static_debug/pkg target/wasm32-unknown-unknown/debug/culsynth_web.wasm
ln -sf `pwd`/static/*.html `pwd`/static/*.js static_debug