#[cfg(all(target_arch = "wasm32", feature = "engine-quickjs"))]
#[path = "macros/pass_class.rs"]
pub mod pass_class;

#[cfg(all(target_arch = "wasm32", feature = "engine-quickjs"))]
#[path = "macros/pass_convert.rs"]
pub mod pass_convert;

#[cfg(all(target_arch = "wasm32", feature = "engine-quickjs"))]
#[path = "macros/pass_method.rs"]
pub mod pass_method;

#[cfg(all(target_arch = "wasm32", feature = "engine-quickjs"))]
#[path = "macros/pass_module.rs"]
pub mod pass_module;

#[cfg(all(target_arch = "wasm32", feature = "engine-quickjs"))]
#[path = "macros/pass_nested_class.rs"]
pub mod pass_nested_class;

#[cfg(all(target_arch = "wasm32", feature = "engine-quickjs"))]
#[path = "macros/pass_trace.rs"]
pub mod pass_trace;

#[cfg(all(feature = "macro", feature = "engine-quickjs"))]
mod macro_tests {
    #[cfg(target_arch = "wasm32")]
    use crate::{
        pass_class, pass_convert, pass_method, pass_module, pass_nested_class, pass_trace,
    };

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn macros() {
        let t = trybuild::TestCases::new();
        t.pass("tests/macros/pass_*.rs");
        #[cfg(feature = "compile-tests")]
        t.compile_fail("tests/compile_fail/*.rs");
        #[cfg(all(feature = "futures", feature = "compile-tests"))]
        t.compile_fail("tests/async_compile_fail/*.rs");
        #[cfg(all(feature = "futures", feature = "parallel", feature = "compile-tests"))]
        t.compile_fail("tests/async_parallel_compile_fail/*.rs");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macros_pass_class() {
        pass_class::main();
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macros_pass_convert() {
        pass_convert::main();
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macros_pass_method() {
        pass_method::main();
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macros_pass_module() {
        pass_module::main();
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macros_pass_nested_class() {
        pass_nested_class::main();
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macros_pass_trace() {
        pass_trace::main();
    }
}
