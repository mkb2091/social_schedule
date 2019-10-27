set -e
cargo make build_release
echo "Optimising WASM"
wasm-opt -O3 pkg/package_bg.wasm -o pkg/package_bg.wasm
echo "Optimising JavaScript"
sed -i '/export default init;/d' pkg/package.js
python3 -c "with open('pkg/package_bg.wasm', 'rb') as file:print('init(Uint8Array.from(%s));' % list(file.read()))" >> pkg/package.js
terser pkg/package.js -c -m > pkg/package_temp.js
mv pkg/package_temp.js pkg/package.js
echo "Generating index.html"
python3 -c "print(open('src/index.html').read().format(js=open('pkg/package.js').read()))" > index.html
