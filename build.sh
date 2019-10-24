set -e
cargo make build_release
echo "Optimising WASM"
wasm-opt -O4 pkg/package_bg.wasm -o pkg/package_bg.wasm
echo "Optimising JavaScript"
terser pkg/package.js -c -m > pkg/package_temp.js 
mv pkg/package_temp.js pkg/package.js 
echo "Generating index.html"
python3 generate_html.py
