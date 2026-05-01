# esabi-sys

[![github](https://img.shields.io/badge/github-delskayn/esabi-8da0cb.svg?style=for-the-badge&logo=github)](https://github.com/DelSkayn/esabi)
[![crates](https://img.shields.io/crates/v/esabi--sys.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/esabi-sys)
[![docs](https://img.shields.io/badge/docs.rs-esabi--sys-66c2a5?style=for-the-badge)](https://docs.rs/esabi-sys)
[![status](https://img.shields.io/github/actions/workflow/status/DelSkayn/esabi/ci.yml?branch=master&style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/DelSkayn/esabi/actions?query=workflow%3ARust)

This crate is a low level unsafe raw bindings for the [QuickJS](https://bellard.org/quickjs/) JavaScript engine.

__NOTE:__ Usually you shouldn't use this crate directly, instead use the top-level [esabi](https://crates.io/crates/esabi) crate which provides high-level safe bindings.

## Patches

In order to fix bugs and get support for some unimplemented features the series of patches applies to released sources.

Hot fixes:
- Fix for _check stack overflow_ (important for Rust)
- Atomic support for `JS_NewClassID` (important for Rust)
- Infinity handling (replacement `1.0 / 0.0` to `INFINITY` constant)

Special patches:
- Reading module exports (`exports` feature)
- Reset stack function (`parallel` feature)
- MSVC support

## Environment Variables

The following environment variables can be used to control which WASI SDK is using during the build:

- `WASI_SDK`: Path to the WASI SDK to use during the build. If unset, this crate will download it automatically instead.
- `RQUICKJS_SYS_NO_WASI_SDK`: If set to `1`, this crate will not attempt to use the WASI SDK
  and instead the `CC`, `AR`, and `CFLAGS` environment variables must be appropriately set.
