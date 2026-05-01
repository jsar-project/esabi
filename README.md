# esabi

[![github](https://img.shields.io/badge/github-delskayn/esabi-8da0cb.svg?style=for-the-badge&logo=github)](https://github.com/DelSkayn/esabi)
[![crates](https://img.shields.io/crates/v/esabi.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/esabi)
[![docs](https://img.shields.io/badge/docs.rs-esabi-66c2a5?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/esabi)
[![status](https://img.shields.io/github/actions/workflow/status/DelSkayn/esabi/ci.yml?branch=master&style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/DelSkayn/esabi/actions/workflows/ci.yml)

This library is a high level bindings of the [QuickJS-NG](https://quickjs-ng.github.io/quickjs/) JavaScript engine, a fork of the [QuickJS](https://bellard.org/quickjs/) Javascript engine.
Its goal is to be an easy to use, and safe wrapper similar to the rlua library.

**QuickJS** is a small and embeddable JavaScript engine. It supports the _ES2020_ specification including modules, asynchronous generators, proxies and BigInt.
It optionally supports mathematical extensions such as big decimal floating point numbers (BigDecimal), big binary floating point numbers (BigFloat) and operator overloading.

## Main features of QuickJS

- Small and easily embeddable: just a few C files, no external dependency, 210 KiB of x86 code for a simple hello world program.
- Fast interpreter with very low startup time: runs the 75000 tests of the ECMAScript Test Suite in about 100 seconds on a single core of a desktop PC.
  The complete life cycle of a runtime instance completes in less than 300 microseconds.
- Almost complete ES2020 support including modules, asynchronous generators and full Annex B support (legacy web compatibility).
- Passes nearly 100% of the ECMAScript Test Suite tests when selecting the ES2020 features. A summary is available at Test262 Report.
- Can compile JavaScript sources to executables with no external dependency.
- Garbage collection using reference counting (to reduce memory usage and have deterministic behavior) with cycle removal.
- Mathematical extensions: BigDecimal, BigFloat, operator overloading, bigint mode, math mode.
- Command line interpreter with contextual colorization implemented in JavaScript.
- Small built-in standard library with C library wrappers.

## Features provided by this crate

- Full integration with async Rust
  - The ES6 Promises can be handled as Rust futures and vice versa
  - Easy integration with almost any async runtime or executor
- Flexible data conversion between Rust and JS
  - Many widely used Rust types can be converted to JS and vice versa
- Support for user-defined allocators
  - The `Runtime` can be created using custom allocator
  - Using Rust's global allocator is also fully supported
- Support for user-defined module resolvers and loaders which also
  can be combined to get more flexible solution for concrete case
- Support for bundling JS modules as a bytecode using `embed` macro
- Support for deferred calling of JS functions
- Full support of ES6 classes
  - Rust data types can be represented as JS classes
  - Data fields can be accessed via object properties
  - Both static and instance members is also supported
  - The properties can be defined with getters and setters
  - Support for constant static properties
  - Support for holding references to JS objects
    (Data type which holds refs should implement `Trace` trait to get garbage collector works properly)
  - Support for extending defined classes by JS

## Community development

This crate doesn't aim to provide system and web APIs. The QuickJS library is close to [V8](https://v8.dev/) in that regard.
If you need APIs from [WinterGC](https://wintercg.org/) or [Node](https://nodejs.org/api/), then you can take a look at the follow community projects:

- [AWS LLRT Modules](https://github.com/awslabs/llrt/tree/main/llrt_modules): Collection of modules that micmic some of the `Node` APIs in pure Rust
- [Extra Modules](https://github.com/esabi/rquickjs-extra): Collection of modules that complement `AWS LLRT Modules` in pure Rust

The community has also built various utilities which might be relevant to you:

- [Serde Integration](https://github.com/esabi/rquickjs-serde): Serde serializer and deserializer for esabi `Value`

## Development status

This bindings is feature complete, mostly stable and ready to use.
The error handling is only thing which may change in the future.
Some experimental features like `parallel` may not works as expected. Use it for your own risk.

## Engine selection

By default, `esabi` keeps using QuickJS:

```toml
esabi = "0.11"
```

This preserves the existing non-wasm behavior and feature layout.

For `wasm32-unknown-unknown`, the recommended path is to disable default features and
explicitly select the Boa backend instead of the default QuickJS backend:

```toml
esabi = { version = "0.11", default-features = false, features = ["std", "engine-boa"] }
```

Example verification commands:

```bash
cargo test --test boa_smoke --no-default-features --features std,engine-boa
cargo check --target wasm32-unknown-unknown --no-default-features --features std,engine-boa
cargo test --target wasm32-unknown-unknown --test boa_smoke --no-default-features --features std,engine-boa --no-run
```

Current Boa backend scope is still compatibility-focused, but it now covers more than the
original runtime skeleton:

- `Runtime`, `Context` and `eval`
- basic `Value` / `Object` conversion and global access
- common `Function::new` signatures and `function::Func`
- `Persistent`
- `Promise`, `PromiseState` and `execute_pending_job()` smoke coverage
- `Exception` / `throw` / `catch`
- `ModuleDef`-backed synthetic modules and fixture-backed imports
- a minimal `Class` slice covering constructor-style wrappers, instance recovery and `this`-based methods

The repository now also contains a browser demo under `web/`:

- `web/runner` exposes a wasm-friendly Boa playground runner
- `web/playground` provides a static UI for examples, support status and `ModuleDef` imports

Known limitations on the Boa backend at the moment:

- `wasm32-unknown-unknown` currently assumes a JavaScript host and uses `getrandom`'s
  `wasm_js` backend under the hood.
- `Object::remove` is not implemented yet.
- `Ctx::eval_with_options(... promise: true)` is not supported yet.
- The current Boa module path supports built-in `ModuleDef` registration for synthetic modules,
  but does not yet provide full QuickJS loader/resolver parity.
- `Class` support is currently partial: native and macro-based classes cover constructor, instance
  methods, accessors, `rename_all`, symbol-named members, descriptor checks, `Ctx`-accepting
  methods that return objects, and a small static-member slice, but lifetime-bearing class fields
  still trail broader QuickJS parity.
- Runtime tuning APIs such as `run_gc` and stack sizing still expose only minimal compatibility behavior.
- QuickJS-specific features such as native `qjs` bindings and async runtime integration still
  require the default QuickJS backend.
- `ArrayBuffer` / `TypedArray` and macro-based APIs are available on the Boa path in a minimal
  compatibility slice, but they do not yet cover full QuickJS parity.

## Supported platforms

Rquickjs needs to compile a C-library which has it's own limitation on supported platforms, furthermore it needs to generate bindings for that platform.
As a result esabi might not compile on all platforms which rust supports.
In general you can allways try to compile esabi with the `bindgen` feature, this should work for most platforms.
Rquickjs ships bindings for a limited set of platforms, for these platforms you don't have to enable the `bindgen` feature.
See below for a list of supported platforms.

| **platform**                   | **shipped bindings** | **tested** | **supported by quickjs** |
| ------------------------------ | :------------------: | :--------: | :----------------------: |
|                                |                      |            |                          |
| x86_64-unknown-linux-gnu       |          ✅          |     ✅     |            ✅            |
| i686-unknown-linux-gnu         |          ✅          |     ✅     |            ✅            |
| aarch64-unknown-linux-gnu      |          ✅          |     ✅     |            ✅            |
| loongarch64-unknown-linux-gnu  |          ✅          |     ✅     |            ✅            |
| x86_64-unknown-linux-musl      |          ✅          |     ✅     |            ✅            |
| aarch64-unknown-linux-musl     |          ✅          |     ✅     |            ✅            |
| loongarch64-unknown-linux-musl |          ✅          |     ✅     |            ✅            |
| x86_64-pc-windows-gnu          |          ✅          |     ✅     |            ✅            |
| i686-pc-windows-gnu            |          ✅          |     ✅     |            ✅            |
| x86_64-pc-windows-msvc         |          ✅          |     ✅     |     ❌ experimental!     |
| aarch64-pc-windows-msvc        |          ✅          |     ❌     |     ❌ experimental!     |
| x86_64-apple-darwin            |          ✅          |     ✅     |            ✅            |
| aarch64-apple-darwin           |          ✅          |     ❌     |            ✅            |
| wasm32-unknown-unknown         |     via `engine-boa` |    smoke   |           N/A            |
| wasm32-wasip1                  |          ✅          |     ✅     |            ✅            |
| wasm32-wasip2                  |          ✅          |     ✅     |            ✅            |
| other                          |          ❌          |     ❌     |         Unknown          |

## License

This library is licensed under the [MIT License](LICENSE)
