#!/bin/bash
set -e
cd `dirname $0`
mkdir -p static
wasm-pack build --target web --no-typescript --out-dir ../static/pkg culsynth_plugin --no-default-features --features "audioworklet"
wasm-pack build --target web --no-typescript --out-dir ../static/pkg culsynth_web
