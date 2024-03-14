#!/bin/bash

cd "$(dirname "$0")/.."
set -e
wasm-pack build --dev --target web
cp assets/index.html pkg
cd assets
npx tailwindcss -i style.css -o ../pkg/style.css
cd ../pkg
python3 -m http.server 8000
