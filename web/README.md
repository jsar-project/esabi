# Boa Web Playground

`web/` contains a browser-facing demo for the Boa backend.

## What It Shows

- Current Boa backend support matrix
- Interactive module-based playground
- Shared parity fixtures backed by Rust `ModuleDef` exports
- Dedicated examples for both partial and blocked APIs such as class methods and typed arrays

## Layout

- `runner/`: wasm entry crate that exposes the Boa playground APIs
- `playground/`: static browser UI
- `../fixtures/boa-parity/examples/`: shared fixture modules used by both the UI and Rust regressions

## Build The Runner

Build the wasm bundle with `wasm-bindgen` or `wasm-pack`.

Example using `wasm-bindgen` CLI:

```bash
cargo build -p boa-playground-runner --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/boa_playground_runner.wasm \
  --target web \
  --out-dir web/dist/runner
```

## Run The Playground

Serve the repository root or the `web/` directory over HTTP, then open:

```text
web/playground/index.html
```

For example:

```bash
python3 -m http.server 8000
```

Then visit `http://localhost:8000/web/playground/`.

## Notes

- The playground targets `wasm32-unknown-unknown + engine-boa`
- `ModuleDef` support is implemented through Boa synthetic modules
- Loader parity with QuickJS is still partial in this demo
- Some examples intentionally fail to document current Boa blockers in an executable way
