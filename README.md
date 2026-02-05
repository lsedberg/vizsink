# Vizsink

## Build

To compile the Vizsink binary, you must first compile the frontend WebAssembly.

Then compile the binary.

### WASM

To build the WebAssembly module, ensure you have `wasm-pack` installed on your system. Then run:

```bash
cd vizsink-wasm
wasm-pack build --target web --out-dir ../vizsink-bin/static/pkg
```

This places the compiled WebAssembly files in the `vizsink-bin/static/pkg` directory for use in the web server.

### Binary

#### MUSL

`x86_64-unknown-linux-musl` seems to work in more environments than `x86_64-unknown-linux-gnu` due to it being less dependendt of system packages.

The binary should be fully self-contained for portability and use in e.g. devcontainers.

```bash
cd vizsink
cargo build --target x86_64-unknown-linux-musl --release
```
