# shackle-wasm

Browser entry point for Shackle’s first Playground integration. It accepts a
selected `.mzn` file and a map of flat, textual Playground files, then returns
either input-only MiniZinc output or structured diagnostics. Data files are
intentionally not compiler inputs in this initial API.

Build a web package with an Edge MiniZinc library:

```sh
MZN_STDLIB_DIR=/path/to/share/minizinc \
  wasm-pack build crates/shackle-wasm --target web --out-dir pkg
```

`MZN_STDLIB_DIR` must be the directory containing `std/stdlib.mzn`, not its
`std` child. Both the upstream library and Shackle’s `share/minizinc` tree are
compiled into the artifact; no browser-time library fetch is used.
