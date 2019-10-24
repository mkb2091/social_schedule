with open('src/index.html') as file:
    data = file.read()

with open('pkg/package.js') as file:
    js = file.read()

with open('pkg/package_bg.wasm', 'rb') as file:
    wasm = file.read()

with open('index.html', 'w') as file:
    file.write(data.format(js=js, wasm=str(list(wasm)).replace(' ', '')))
