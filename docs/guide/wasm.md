# WASM Support

`rquickjs` now supports `wasm32-unknown-unknown` as a browser-oriented target.

## What Works

- `Runtime` and `Context`
- Macro-powered APIs
- In-memory module resolution and loading
- Builtin module bundles and direct module declarations

## What Is Intentionally Excluded

- `FileResolver`
- `ScriptLoader`
- `NativeLoader`
- Native dynamic library loading

## Browser Playground Model

The browser playground uses a thin wasm bridge that owns the runtime and exposes a small API to the frontend:

- `init()`
- `list_samples()`
- `load_sample()`
- `run()`
- `reset()`

## Structured Errors

The structured error payload described here is primarily the contract for the browser-hosted wasm playground bridge under `docs/wasm` and the frontend consumer under `docs/playground-app`. It is not a blanket guarantee for every wasm embedding or every raw `wasm-bindgen` export. In particular, `init()`, `reset()`, `list_samples()`, and `load_sample()` may still fail by throwing a plain `JsValue` instead of returning the `run()` payload documented below.

`run()` returns a structured payload instead of only surfacing a raw exception string. Successful runs include the selected mode, captured `stdout`/`stderr`, and the final result value. Failed runs return an `error` object with fields such as:

- `kind`
- `name`
- `message`
- `stack` when the engine provides one

`stack` is intentionally optional. The browser playground can legitimately return an error payload without it on paths such as:

- JavaScript throws a non-`Error` value like `throw 'plain failure'`, which becomes a `ThrownValue` payload and usually has no stack to surface.
- JavaScript throws an object that does not define a `stack` property.
- The wasm host or Rust bridge reports an internal failure and the error is mapped to `kind: "internal"`.
- Source parsing, module loading, or other failures that happen before QuickJS materializes a useful stack trace.

## Maintainer Validation

Use the following repeatable checks when touching the playground error contract or its rendering:

1. Run `cargo test --manifest-path docs/wasm/Cargo.toml`.
2. Run `node --test docs/playground-app/src/display.test.ts`.
3. Run `npm run build` from `docs/` to rebuild the wasm bundle and the VuePress site.
4. Start `npm run dev` from `docs/` and open `/playground/` in a browser for manual validation.

Recommended manual samples:

- Built-in `error-demo`: confirms the normal runtime exception path returns `kind: "execution"`, `name: "TypeError"`, `message: "Playground demo failure"`, and a non-empty `stack`.
- Edited script `throw 'plain failure'`: confirms the structured payload still renders, but `stack` can be missing and the UI falls back to `No stack.`.
- Edited script with malformed JavaScript such as `function broken(`: confirms parse-time failures still return a structured error payload and may omit `stack`.
