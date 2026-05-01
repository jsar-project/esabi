//! # High-level bindings to QuickJS
//!
//! The `esabi` crate provides safe high-level bindings to the
//! [QuickJS](https://bellard.org/quickjs/) JavaScript engine and keeps QuickJS as the default
//! backend. This crate is heavily inspired by the [rlua](https://crates.io/crates/rlua) crate.
//!
//! For `wasm32-unknown-unknown`, you can switch to the `engine-boa` backend. The Boa path is
//! intentionally smaller than the default QuickJS backend and currently focuses on `Runtime`,
//! `Context`, basic value conversion, object access and `eval`.

#![allow(unknown_lints)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::uninlined_format_args)]
#![allow(mismatched_lifetime_syntaxes)]
#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]
#![allow(clippy::doc_lazy_continuation)]
#![cfg_attr(not(test), no_std)]

#[cfg(all(feature = "engine-quickjs", feature = "engine-boa"))]
compile_error!("Only one JavaScript engine can be enabled at a time. Choose either `engine-quickjs` or `engine-boa`.");

#[cfg(not(any(feature = "engine-quickjs", feature = "engine-boa")))]
compile_error!("No JavaScript engine selected. Enable `engine-quickjs` or `engine-boa`.");

#[doc(hidden)]
pub extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

pub(crate) use alloc::string::String as StdString;
#[cfg(feature = "engine-quickjs")]
pub(crate) use core::result::Result as StdResult;

#[cfg(feature = "engine-boa")]
mod boa_backend;
pub(crate) mod engine;
mod js_lifetime;
pub mod markers;
#[cfg(feature = "engine-quickjs")]
mod persistent;
#[cfg(feature = "engine-quickjs")]
mod result;
#[cfg(feature = "engine-quickjs")]
mod safe_ref;
#[cfg(feature = "engine-quickjs")]
mod util;
#[cfg(feature = "engine-quickjs")]
mod value;
#[cfg(feature = "engine-quickjs")]
pub(crate) use safe_ref::*;
#[cfg(feature = "engine-quickjs")]
pub mod runtime;
#[cfg(feature = "engine-quickjs")]
pub use runtime::Runtime;
#[cfg(feature = "engine-quickjs")]
pub mod context;
#[cfg(feature = "engine-quickjs")]
pub use context::{Context, Ctx};
#[cfg(feature = "engine-quickjs")]
pub mod class;
#[cfg(feature = "engine-quickjs")]
pub use class::Class;
pub use js_lifetime::JsLifetime;
#[cfg(feature = "engine-quickjs")]
pub use persistent::Persistent;
#[cfg(feature = "engine-quickjs")]
pub use result::{CatchResultExt, CaughtError, CaughtResult, Error, Result, ThrowResultExt};
#[cfg(feature = "engine-quickjs")]
pub use value::{
    array, atom, convert, function, module, object, promise, proxy, Array, Atom, BigInt, CString,
    Coerced, Constructor, Exception, Filter, FromAtom, FromIteratorJs, FromJs, Function, IntoAtom,
    IntoJs, IteratorJs, Module, Null, Object, Promise, Proxy, String, Symbol, Type, Undefined,
    Value, WriteOptions, WriteOptionsEndianness,
};

#[cfg(feature = "engine-boa")]
pub mod atom {
    pub use crate::boa_backend::atom::PredefinedAtom;
}

#[cfg(feature = "engine-boa")]
pub mod class {
    pub use crate::boa_backend::class::{
        impl_, Class, JsClass, Mutability, OwnedBorrow, OwnedBorrowMut, Readable, Trace, Tracer,
        Writable,
    };
}

#[cfg(feature = "engine-boa")]
pub mod context {
    pub use crate::boa_backend::context::{
        intrinsic, Context, ContextBuilder, Ctx, EvalOptions, Intrinsic,
    };
}

#[cfg(feature = "engine-boa")]
pub mod runtime {
    pub use crate::boa_backend::runtime::{Runtime, WeakRuntime};
}

#[cfg(feature = "engine-boa")]
pub mod value {
    pub use crate::boa_backend::{
        array_buffer::{ArrayBuffer, RawArrayBuffer},
        exception::Exception,
        typed_array::{TypedArray, TypedArrayItem},
        value::{FromJs, IntoJs, Null, Object, Type, Undefined, Value},
    };
}

#[cfg(feature = "engine-boa")]
pub mod function {
    pub use crate::boa_backend::function::{
        from_js_func, into_function_value, new_class_from_js_func, Constructor, Exhaustive, Flat,
        FromParam, Func, FuncArg, Function, IntoArgs, IntoJsFunc, IntoFunctionValue, MutFn,
        OnceFn, Opt, ParamRequirement, Params, ParamsAccessor, Rest, This,
    };
}

#[cfg(feature = "engine-boa")]
pub mod object {
    pub use crate::boa_backend::object::{Accessor, AsProperty, Property};
}

#[cfg(feature = "engine-boa")]
pub mod module {
    pub use crate::boa_backend::module::{
        Declared, Declarations, Evaluated, Exports, Module, ModuleDef, ModuleLoadFn,
    };
}

#[cfg(all(feature = "loader", feature = "engine-boa"))]
pub mod loader {
    pub use crate::boa_backend::loader::{ImportAttributes, Loader, Resolver};
}

#[cfg(feature = "engine-boa")]
pub mod persistent {
    pub use crate::boa_backend::persistent::Persistent;
}

#[cfg(feature = "engine-boa")]
pub mod promise {
    pub use crate::boa_backend::promise::{Promise, PromiseState};
}

#[cfg(feature = "engine-boa")]
pub use boa_backend::{
    atom::PredefinedAtom,
    class::{Class, JsClass, Trace, Tracer},
    context::{Context, ContextBuilder, Ctx, EvalOptions, Intrinsic},
    exception::Exception,
    function::{
        from_js_func, into_function_value, new_class_from_js_func, Constructor, Exhaustive, Flat,
        FromParam, Func, FuncArg, Function, IntoArgs, IntoJsFunc, IntoFunctionValue, MutFn,
        OnceFn, Opt, ParamRequirement, Params, ParamsAccessor, Rest, This,
    },
    module::{Declared, Declarations, Evaluated, Exports, Module, ModuleDef, ModuleLoadFn},
    object::{Accessor, AsProperty, Property},
    persistent::Persistent,
    promise::{Promise, PromiseState},
    result::{Error, Result},
    runtime::{MemoryUsage, Runtime, WeakRuntime},
    array_buffer::{ArrayBuffer, RawArrayBuffer},
    typed_array::{TypedArray, TypedArrayItem},
    value::{FromJs, IntoJs, Null, Object, Type, Undefined, Value},
};

#[cfg(feature = "engine-quickjs")]
pub mod allocator;
#[cfg(all(feature = "loader", feature = "engine-quickjs"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "loader")))]
pub mod loader;

#[cfg(all(feature = "futures", feature = "engine-quickjs"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "futures")))]
pub use context::AsyncContext;
#[cfg(all(feature = "multi-ctx", feature = "engine-quickjs"))]
pub use context::MultiWith;
#[cfg(all(feature = "futures", feature = "engine-quickjs"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "futures")))]
pub use runtime::AsyncRuntime;
#[cfg(feature = "engine-quickjs")]
pub use value::{ArrayBuffer, Iterable, IterableFn, JsIterator, TypedArray};

#[cfg(feature = "engine-quickjs")]
pub mod qjs {
    //! Native low-level bindings
    pub use esabi_sys::*;
}

#[cfg(feature = "phf")]
#[doc(hidden)]
pub mod phf {
    pub use phf::*;
}

pub mod prelude {
    //! A group of often used types.
    #[cfg(all(feature = "multi-ctx", feature = "engine-quickjs"))]
    pub use crate::context::MultiWith;
    #[cfg(feature = "engine-quickjs")]
    pub use crate::{
        context::Ctx,
        convert::{Coerced, FromAtom, FromIteratorJs, FromJs, IntoAtom, IntoJs, IteratorJs, List},
        function::{
            Exhaustive, Flat, Func, FuncArg, IntoArg, IntoArgs, MutFn, OnceFn, Opt, Rest, This,
        },
        result::{CatchResultExt, ThrowResultExt},
        JsLifetime,
    };
    #[cfg(feature = "engine-boa")]
    pub use crate::{
        ArrayBuffer,
        class::{Class, JsClass},
        context::Ctx,
        function::{Constructor, Func},
        Exception, FromJs, IntoJs, JsLifetime, Object, TypedArray, Value,
    };
    #[cfg(all(feature = "futures", feature = "engine-quickjs"))]
    #[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "futures")))]
    pub use crate::{
        function::Async,
        promise::{Promise, Promised},
    };
}

#[cfg(test)]
pub(crate) fn test_with<F, R>(func: F) -> R
where
    F: FnOnce(Ctx) -> R,
{
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();
    ctx.with(func)
}

mod deprecated_features {
    #[cfg(feature = "properties")]
    #[allow(unused_imports)]
    use properties as _;
    #[cfg(feature = "properties")]
    #[deprecated(
        note = "The esabi crate feature `properties` is deprecated, the functionality it provided is now enabled by default.
To remove this warning remove the use of the feature when specifying the dependency."
    )]
    mod properties {}

    #[cfg(feature = "array-buffer")]
    #[allow(unused_imports)]
    use array_buffer as _;
    #[cfg(feature = "array-buffer")]
    #[deprecated(
        note = "The esabi crate feature `array-buffer` is deprecated, the functionality it provided is now enabled by default.
To remove this warning remove the use of the feature when specifying the dependency."
    )]
    mod array_buffer {}

    #[cfg(feature = "classes")]
    #[allow(unused_imports)]
    use classes as _;
    #[cfg(feature = "classes")]
    #[deprecated(
        note = "The esabi crate feature `classes` is deprecated, the functionality it provided is now enabled by default.
To remove this warning remove the use of the feature when specifying the dependency."
    )]
    mod classes {}

    #[cfg(feature = "allocator")]
    #[allow(unused_imports)]
    use allocator as _;
    #[cfg(feature = "allocator")]
    #[deprecated(
        note = "The esabi crate feature `allocator` is deprecated, the functionality it provided is now enabled by default.
To remove this warning remove the use of the feature when specifying the dependency."
    )]
    mod allocator {}
}
