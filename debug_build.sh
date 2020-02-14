set -e
cargo make build
sed -i '/export default init;/d' pkg/package.js
python3 -c "with open('pkg/package_bg.wasm', 'rb') as file:print('init(Uint8Array.from(%s));' % list(file.read()))" >> pkg/package.js
echo "Generating index.html"
python3 -c "print(open('src/index.html').read().format(js=open('pkg/package.js').read()))" > index.html
