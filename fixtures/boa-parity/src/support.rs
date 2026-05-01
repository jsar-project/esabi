use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SupportEntry {
    pub name: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
}

static SUPPORT_MATRIX: &[SupportEntry] = &[
    SupportEntry {
        name: "Runtime / Context / eval",
        status: "supported",
        detail: "Boa backend can create runtimes, contexts and evaluate JavaScript.",
    },
    SupportEntry {
        name: "globals / Value / Object",
        status: "supported",
        detail: "Basic value conversion and global object access are available.",
    },
    SupportEntry {
        name: "Function / Func",
        status: "supported",
        detail: "Fixture-backed closures cover common Function::new signatures and function::Func exports.",
    },
    SupportEntry {
        name: "Persistent / Promise / jobs",
        status: "supported",
        detail: "Shared fixtures and smoke tests cover saved handles, Promise state, job execution and rejection propagation.",
    },
    SupportEntry {
        name: "Exception / throw / catch",
        status: "supported",
        detail: "Boa exposes the compatibility layer used by fixture-backed error modules and smoke tests.",
    },
    SupportEntry {
        name: "ModuleDef / import",
        status: "supported",
        detail: "Fixture-backed synthetic modules can be declared and imported through Boa.",
    },
    SupportEntry {
        name: "Loader / resolver parity",
        status: "partial",
        detail: "Custom loaders work for the current module chain, but import attributes and full QuickJS parity are still missing.",
    },
    SupportEntry {
        name: "Runtime tuning APIs",
        status: "partial",
        detail: "memory_usage exists, but run_gc, max stack sizing and related controls still expose only minimal compatibility behavior.",
    },
    SupportEntry {
        name: "Class / methods parity",
        status: "partial",
        detail: "Boa now supports native and macro-based class flows with constructor, instance methods, accessors, rename_all handling, symbol-named members, descriptor checks, Ctx-returning methods and a small static-member slice; lifetime-bearing class fields still trail broader parity.",
    },
    SupportEntry {
        name: "ArrayBuffer / TypedArray parity",
        status: "partial",
        detail: "Minimal ArrayBuffer-backed typed array creation, conversion and byte-view access now work on Boa, but the broader helper surface still trails QuickJS parity.",
    },
    SupportEntry {
        name: "QuickJS helper APIs",
        status: "blocked",
        detail: "QuickJS-specific helpers such as Constructor, Opt/Rest and value inspection helpers are not available on the Boa path yet.",
    },
];

pub fn support_matrix() -> &'static [SupportEntry] {
    SUPPORT_MATRIX
}
