# esabi-core

[![github](https://img.shields.io/badge/github-delskayn/esabi-8da0cb.svg?style=for-the-badge&logo=github)](https://github.com/DelSkayn/esabi)
[![crates](https://img.shields.io/crates/v/esabi--core.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/esabi-core)
[![docs](https://img.shields.io/badge/docs.rs-esabi--core-66c2a5?style=for-the-badge)](https://docs.rs/esabi-core)
[![status](https://img.shields.io/github/actions/workflow/status/DelSkayn/esabi/ci.yml?branch=master&style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/DelSkayn/esabi/actions?query=workflow%3ARust)

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

## Development status

This bindings is feature complete, mostly stable and ready to use.
The error handling is only thing which may change in the future.
Some experimental features like `parallel` may not works as expected. Use it for your own risk.

## Engine selection

`esabi-core` now defaults to the QuickJS backend and keeps the existing desktop/server path
unchanged unless you explicitly switch engines.

For `wasm32-unknown-unknown`, use the Boa backend:

```toml
esabi-core = { version = "0.11", default-features = false, features = ["std", "engine-boa"] }
```

The current Boa path is a compatibility-focused subset that keeps `Runtime`, `Context`, basic
value conversion, object access, `eval` and `ModuleDef`-based synthetic modules available without
requiring the QuickJS C toolchain.

Known limitations on the Boa backend:

- `wasm32-unknown-unknown` currently assumes a JavaScript host and uses `getrandom`'s
  `wasm_js` backend under the hood.
- `Object::remove` is not implemented yet.
- Promise-mode `eval_with_options` is not supported yet.
- Synthetic-module `ModuleDef` support exists, but full QuickJS loader/resolver parity does not.
- QuickJS-only capabilities such as low-level `qjs` bindings, loaders, async runtime support and
  native module loading still require the default QuickJS backend.

## License

This library is licensed under the [MIT License](LICENSE)
