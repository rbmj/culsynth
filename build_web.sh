#!/bin/bash
set -e
mkdir -p static
#PKGDIR=`dirname $0`
#PKGDIR=`realpath $PKGDIR`
#mkdir -p "$PKGDIR/static/pkg"
#cd "$PKGDIR/culsynth_plugin"
wasm-pack build --target web --no-typescript --out-dir ../static/pkg culsynth_plugin --no-default-features --features "audioworklet"
#cd ../culsynth_web
wasm-pack build --target web --no-typescript --out-dir ../static/pkg culsynth_web
cp culsynth_web/*.js culsynth_web/*.html static
