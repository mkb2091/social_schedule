set -e
cargo make build_release
wasm-opt -O4 pkg/package_bg.wasm -o pkg/package_bg.wasm
python3 generate_html.py
