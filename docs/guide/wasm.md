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
