# Vizsink-Wasm

This crate provides WebAssembly bindings for rendering visualizations in the browser using WebSockets to receive data.

## Build

To build the WebAssembly module, ensure you have `wasm-pack` installed. Then run:

```bash
cd vizsink-wasm
wasm-pack build --target web --out-dir ../vizsink-bin/static/pkg
```

This places the compiled WebAssembly files in the `vizsink-bin/static/pkg` directory for use in the web server.
