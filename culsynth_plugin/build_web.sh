#!/bin/bash
set -e
cd `dirname $0`
wasm-pack build --target web --no-default-features --features "audioworklet"
cp ./pkg/culsynth_plugin.js ./pkg/culsynth_plugin_bg.wasm ../culsynth_web/pkg
