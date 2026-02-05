# Vizsink

A command-line tool for visualzing data streams in real-time.

Draw and visualize simple primitives with any program, written in any language,
with minimal boilerplate.

Vizsink is currently work in progress.

![Example of Vizsink used to visualize partitioning and path finding of a drone swarm](docs/assets/vizsink_example_1.png)
Example of Vizsink being used to visualize the partitioning and
path finding of a drone swarm.

## How it works

Most programming langauges allow you to print to the terminal, or rather stdout.
By writing your print statements in a specific format,
such as `draw circle x=1 y=2 r=3`, you can get this visualized on a 2D canvas.
Simply pipe the result of your program into Vizsink.
Vizsink will then start a localhost server
that you can open in your browser to visualize your data.

The server uses websockets to communicate accumulated data when you first connect,
then stream new commands as new lines are read from stdin.

### Streaming over SSH

The visualization is done with a server and browser for easy compatibility.
If you need to visualize data from a program running on a server or similar,
you can forward the ssh port used.

By running the following code, you can open your browser at localhost:8080,
as if you opened a browser on the server.

```bash
ssh -L 8080:localhost:8080 user@server -N
```

## Future Plans

This program is currently not optimized when it comes to both
server streaming and rendering.

The MVP has provided much insight in the does and dont's,
and an improved and optimized version would most likely
require a full rewrite.
Using a more ECS kind of approach has been thought off.

## Build

To compile the Vizsink binary, you must first compile the frontend WebAssembly.

Then compile the binary.

### WASM

To build the WebAssembly module, ensure you have `wasm-pack` installed on your system.
Then run:

```bash
cd vizsink-wasm
wasm-pack build --target web --out-dir ../vizsink-bin/static/pkg
```

This places the compiled WebAssembly files in the `vizsink-bin/static/pkg`
directory for use in the web server.

### Binary

#### MUSL

`x86_64-unknown-linux-musl` seems to work in more environments than `x86_64-unknown-linux-gnu`
due to it being less dependendt of system packages.

The binary should be fully self-contained for portability and use in e.g. devcontainers.

```bash
cd vizsink
cargo build --target x86_64-unknown-linux-musl --release
```
