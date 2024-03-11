#!/bin/bash

cd "$(dirname "$0")/.."
set -e
wasm-pack build --dev --target web
cp assets/index.html pkg
cd pkg
python3 -m http.server 8000
