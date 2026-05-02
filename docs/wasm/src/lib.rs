use std::{cell::RefCell, rc::Rc};

use rquickjs::{
    function::Rest,
    loader::{BuiltinLoader, BuiltinResolver, ModuleLoader},
    module::{Declarations, Exports, ModuleDef},
    Coerced, Context, Ctx, Exception, Function, Module, Object, Result as JsResult, Runtime,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

const BUILTIN_MODULE_NAME: &str = "playground/builtin/loader";
const BUILTIN_MODULE_SOURCE: &str = r#"
export const label = 'module loader';
export function meaning() {
  return 42;
}
"#;
const RUST_CONSTANTS_MODULE_NAME: &str = "playground/rust/constants";
const RUST_FUNCTIONS_MODULE_NAME: &str = "playground/rust/functions";
const RUST_MIXED_MODULE_NAME: &str = "playground/rust/mixed";
const RUST_OBJECT_MODULE_NAME: &str = "playground/rust/object";
const HELLO_SOURCE: &str = r#"
print('Hello from rquickjs');
const doubled = [1, 2, 3].map((value) => value * 2);
JSON.stringify({ doubled, answer: 40 + 2 });
"#;
const CONSOLE_SOURCE: &str = r#"
print('stdout: print');
console.log('stdout: console.log');
console.error('stderr: console.error');
JSON.stringify({ result: 'host apis exercised', stdoutLines: 2, stderrLines: 1 });
"#;
const BUILTIN_LOADER_SAMPLE_SOURCE: &str = r#"
import { label, meaning } from 'playground/builtin/loader';
console.log(`loaded ${label}`);
globalThis.__playgroundResult = `${label} -> ${meaning()}`;
"#;
const RUST_CONSTANTS_SOURCE: &str = r#"
import { moduleName, engine, answer, supportsModules } from 'playground/rust/constants';
globalThis.__playgroundResult = JSON.stringify({ moduleName, engine, answer, supportsModules });
"#;
const RUST_FUNCTIONS_SOURCE: &str = r#"
import { greet, double } from 'playground/rust/functions';
console.log(greet('playground'));
globalThis.__playgroundResult = JSON.stringify({ doubled: double(21), greeting: greet('browser') });
"#;
const RUST_MIXED_SOURCE: &str = r#"
import { label, meaning, multiply } from 'playground/rust/mixed';
globalThis.__playgroundResult = `${label}: ${multiply(meaning, 2)}`;
"#;
const RUST_OBJECT_SOURCE: &str = r#"
import { api } from 'playground/rust/object';
console.log(`api version ${api.version}`);
globalThis.__playgroundResult = JSON.stringify({
  greeting: api.greet('playground'),
  tripled: api.triple(14),
  runtime: api.runtime,
});
"#;
const ERROR_SOURCE: &str = r#"
console.log('About to throw');
throw new TypeError('Playground demo failure');
"#;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum SampleMode {
    Script,
    Module,
}

#[derive(Clone, Copy)]
struct Sample {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    mode: SampleMode,
    source: &'static str,
    notes: &'static [&'static str],
}

#[derive(Serialize)]
struct SampleMeta {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    mode: SampleMode,
}

#[derive(Serialize)]
struct SamplePayload {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    mode: SampleMode,
    source: &'static str,
    notes: &'static [&'static str],
}

#[derive(Serialize)]
struct RunError {
    kind: &'static str,
    name: String,
    message: String,
    stack: Option<String>,
}

#[derive(Serialize)]
struct RunResult {
    ok: bool,
    mode: SampleMode,
    result: Option<String>,
    stdout: Vec<String>,
    stderr: Vec<String>,
    error: Option<RunError>,
}

const SAMPLES: &[Sample] = &[
    Sample {
        id: "hello-world",
        title: "Eval Hello World",
        summary: "Run a script and capture its return value.",
        mode: SampleMode::Script,
        source: HELLO_SOURCE,
        notes: &[
            "Uses the global print helper wired up by the Rust playground bridge.",
            "Returns the last expression as the Result panel output.",
        ],
    },
    Sample {
        id: "console-and-print",
        title: "Console And Print",
        summary: "Exercise print, console.log, and console.error from the Rust host bridge.",
        mode: SampleMode::Script,
        source: CONSOLE_SOURCE,
        notes: &[
            "Shows stdout and stderr flowing through the browser host wiring.",
            "Returns a JSON payload so the output panel has both result and log text.",
        ],
    },
    Sample {
        id: "builtin-loader",
        title: "Builtin Loader",
        summary: "Import an in-memory JavaScript module bundled directly in the wasm runtime.",
        mode: SampleMode::Module,
        source: BUILTIN_LOADER_SAMPLE_SOURCE,
        notes: &[
            "This sample uses BuiltinLoader and BuiltinResolver with a JavaScript source string.",
            "Assign to globalThis.__playgroundResult to surface a result string.",
        ],
    },
    Sample {
        id: "rust-moduledef-constants",
        title: "Rust ModuleDef: Constants",
        summary: "Import constants exported by a Rust-defined native module.",
        mode: SampleMode::Module,
        source: RUST_CONSTANTS_SOURCE,
        notes: &[
            "The module is declared from Rust through ModuleDef, not from a JavaScript source string.",
            "Exports strings, numbers, and booleans into JavaScript imports.",
        ],
    },
    Sample {
        id: "rust-moduledef-functions",
        title: "Rust ModuleDef: Functions",
        summary: "Call Rust-defined exported functions from an ES module import.",
        mode: SampleMode::Module,
        source: RUST_FUNCTIONS_SOURCE,
        notes: &[
            "Demonstrates Rust closures exported as callable JavaScript module members.",
            "Uses both console output and a structured result value.",
        ],
    },
    Sample {
        id: "rust-moduledef-mixed",
        title: "Rust ModuleDef: Mixed Exports",
        summary: "Mix constant and function exports inside one Rust native module.",
        mode: SampleMode::Module,
        source: RUST_MIXED_SOURCE,
        notes: &[
            "Combines a static value with a callable export from the same ModuleDef.",
            "Useful for modules that expose metadata alongside behavior.",
        ],
    },
    Sample {
        id: "rust-moduledef-object",
        title: "Rust ModuleDef: Object API",
        summary: "Export a Rust-built object with fields and methods through ModuleDef.",
        mode: SampleMode::Module,
        source: RUST_OBJECT_SOURCE,
        notes: &[
            "Exports an object value built in Rust, including nested methods.",
            "Shows how a native module can feel like a small JavaScript API surface.",
        ],
    },
    Sample {
        id: "error-demo",
        title: "Structured Error",
        summary: "Show structured syntax and runtime exception handling.",
        mode: SampleMode::Script,
        source: ERROR_SOURCE,
        notes: &["Run this sample to verify stack capture and error rendering."],
    },
];

struct Playground {
    _runtime: Runtime,
    context: Context,
    stdout: Rc<RefCell<Vec<String>>>,
    stderr: Rc<RefCell<Vec<String>>>,
}

struct RustConstantsModule;

impl ModuleDef for RustConstantsModule {
    fn declare(declare: &Declarations) -> JsResult<()> {
        declare.declare("moduleName")?;
        declare.declare("engine")?;
        declare.declare("answer")?;
        declare.declare("supportsModules")?;
        Ok(())
    }

    fn evaluate<'js>(_ctx: &Ctx<'js>, exports: &Exports<'js>) -> JsResult<()> {
        exports.export("moduleName", "rust constants")?;
        exports.export("engine", "rquickjs")?;
        exports.export("answer", 42)?;
        exports.export("supportsModules", true)?;
        Ok(())
    }
}

struct RustFunctionsModule;

impl ModuleDef for RustFunctionsModule {
    fn declare(declare: &Declarations) -> JsResult<()> {
        declare.declare("greet")?;
        declare.declare("double")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> JsResult<()> {
        exports.export(
            "greet",
            Function::new(ctx.clone(), |name: String| {
                format!("Hello from Rust, {name}!")
            })?
            .with_name("greet")?,
        )?;
        exports.export(
            "double",
            Function::new(ctx.clone(), |value: i32| value * 2)?.with_name("double")?,
        )?;
        Ok(())
    }
}

struct RustMixedModule;

impl ModuleDef for RustMixedModule {
    fn declare(declare: &Declarations) -> JsResult<()> {
        declare.declare("label")?;
        declare.declare("meaning")?;
        declare.declare("multiply")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> JsResult<()> {
        exports.export("label", "mixed exports")?;
        exports.export("meaning", 21)?;
        exports.export(
            "multiply",
            Function::new(ctx.clone(), |left: i32, right: i32| left * right)?
                .with_name("multiply")?,
        )?;
        Ok(())
    }
}

struct RustObjectModule;

impl ModuleDef for RustObjectModule {
    fn declare(declare: &Declarations) -> JsResult<()> {
        declare.declare("api")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> JsResult<()> {
        let api = Object::new(ctx.clone())?;
        api.set("version", "0.1.0")?;
        api.set("runtime", "wasm playground")?;
        api.set(
            "greet",
            Function::new(ctx.clone(), |name: String| {
                format!("Rust object says hi to {name}")
            })?
            .with_name("greet")?,
        )?;
        api.set(
            "triple",
            Function::new(ctx.clone(), |value: i32| value * 3)?.with_name("triple")?,
        )?;
        exports.export("api", api)?;
        Ok(())
    }
}

thread_local! {
    static PLAYGROUND: RefCell<Option<Playground>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn init() -> Result<JsValue, JsValue> {
    PLAYGROUND.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(Playground::new().map_err(js_error)?);
        }
        Ok(JsValue::from_str("ready"))
    })
}

#[wasm_bindgen]
pub fn reset() -> Result<JsValue, JsValue> {
    PLAYGROUND.with(|slot| {
        let mut slot = slot.borrow_mut();
        *slot = Some(Playground::new().map_err(js_error)?);
        Ok(JsValue::from_str("reset"))
    })
}

#[wasm_bindgen]
pub fn list_samples() -> Result<JsValue, JsValue> {
    to_js(
        &SAMPLES
            .iter()
            .map(|sample| SampleMeta {
                id: sample.id,
                title: sample.title,
                summary: sample.summary,
                mode: sample.mode,
            })
            .collect::<Vec<_>>(),
    )
}

#[wasm_bindgen]
pub fn load_sample(sample_id: String) -> Result<JsValue, JsValue> {
    let sample = sample_by_id(&sample_id)
        .ok_or_else(|| JsValue::from_str(&format!("Unknown sample id: {sample_id}")))?;

    to_js(&SamplePayload {
        id: sample.id,
        title: sample.title,
        summary: sample.summary,
        mode: sample.mode,
        source: sample.source,
        notes: sample.notes,
    })
}

#[wasm_bindgen]
pub fn run(source: String, sample_id: Option<String>) -> Result<JsValue, JsValue> {
    init()?;
    PLAYGROUND.with(|slot| {
        let slot = slot.borrow();
        let playground = slot
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Playground is not initialized"))?;
        playground.run(&source, sample_id.as_deref())
    })
}

impl Playground {
    fn new() -> JsResult<Self> {
        let stdout = Rc::new(RefCell::new(Vec::new()));
        let stderr = Rc::new(RefCell::new(Vec::new()));
        let runtime = Runtime::new()?;
        runtime.set_loader(
            BuiltinResolver::default()
                .with_module(BUILTIN_MODULE_NAME)
                .with_module(RUST_CONSTANTS_MODULE_NAME)
                .with_module(RUST_FUNCTIONS_MODULE_NAME)
                .with_module(RUST_MIXED_MODULE_NAME)
                .with_module(RUST_OBJECT_MODULE_NAME),
            (
                BuiltinLoader::default().with_module(BUILTIN_MODULE_NAME, BUILTIN_MODULE_SOURCE),
                ModuleLoader::default()
                    .with_module(RUST_CONSTANTS_MODULE_NAME, RustConstantsModule)
                    .with_module(RUST_FUNCTIONS_MODULE_NAME, RustFunctionsModule)
                    .with_module(RUST_MIXED_MODULE_NAME, RustMixedModule)
                    .with_module(RUST_OBJECT_MODULE_NAME, RustObjectModule),
            ),
        );
        let context = Context::full(&runtime)?;

        let stdout_handle = Rc::clone(&stdout);
        let stderr_handle = Rc::clone(&stderr);
        context.with(|ctx| install_host_functions(&ctx, stdout_handle, stderr_handle))?;

        Ok(Self {
            _runtime: runtime,
            context,
            stdout,
            stderr,
        })
    }

    fn run(&self, source: &str, sample_id: Option<&str>) -> Result<JsValue, JsValue> {
        self.stdout.borrow_mut().clear();
        self.stderr.borrow_mut().clear();

        let mode = sample_id
            .and_then(sample_by_id)
            .map(|sample| sample.mode)
            .unwrap_or(SampleMode::Script);

        let result = self.context.with(|ctx| {
            let globals = ctx.globals();
            let _ = globals.set("__playgroundResult", ());

            match mode {
                SampleMode::Script => match ctx.eval::<Coerced<String>, _>(source) {
                    Ok(value) => self.success_result(mode, Some(value.0)),
                    Err(error) => self.execution_error(&ctx, error, mode),
                },
                SampleMode::Module => {
                    match Module::evaluate(ctx.clone(), "playground-entry", source) {
                        Ok(promise) => match promise.finish::<()>() {
                            Ok(()) => {
                                let result = globals
                                    .get::<_, Option<Coerced<String>>>("__playgroundResult")
                                    .ok()
                                    .flatten()
                                    .map(|value| value.0)
                                    .or_else(|| Some(String::from("module evaluated")));
                                self.success_result(mode, result)
                            }
                            Err(error) => self.execution_error(&ctx, error, mode),
                        },
                        Err(error) => self.execution_error(&ctx, error, mode),
                    }
                }
            }
        });

        to_js(&result)
    }

    fn success_result(&self, mode: SampleMode, result: Option<String>) -> RunResult {
        RunResult {
            ok: true,
            mode,
            result,
            stdout: self.stdout.borrow().clone(),
            stderr: self.stderr.borrow().clone(),
            error: None,
        }
    }

    fn execution_error(
        &self,
        ctx: &Ctx<'_>,
        error: rquickjs::Error,
        mode: SampleMode,
    ) -> RunResult {
        let exception = ctx.catch().into_object().and_then(Exception::from_object);
        let (name, message, stack) = if let Some(exception) = exception {
            let name = exception
                .as_object()
                .get::<_, Option<Coerced<String>>>("name")
                .ok()
                .flatten()
                .map(|value| value.0)
                .unwrap_or_else(|| String::from("Error"));
            let message = exception.message().unwrap_or_else(|| error.to_string());
            (name, message, exception.stack())
        } else {
            (String::from("InternalError"), error.to_string(), None)
        };

        RunResult {
            ok: false,
            mode,
            result: None,
            stdout: self.stdout.borrow().clone(),
            stderr: self.stderr.borrow().clone(),
            error: Some(RunError {
                kind: "execution",
                name,
                message,
                stack,
            }),
        }
    }
}

fn install_host_functions(
    ctx: &Ctx<'_>,
    stdout: Rc<RefCell<Vec<String>>>,
    stderr: Rc<RefCell<Vec<String>>>,
) -> JsResult<()> {
    let global = ctx.globals();

    let print_stdout = Rc::clone(&stdout);
    let print = Function::new(ctx.clone(), move |args: Rest<Coerced<String>>| {
        print_stdout.borrow_mut().push(join_args(args));
    })?
    .with_name("print")?;
    global.set("print", print)?;

    let console = Object::new(ctx.clone())?;

    let log_stdout = Rc::clone(&stdout);
    console.set(
        "log",
        Function::new(ctx.clone(), move |args: Rest<Coerced<String>>| {
            log_stdout.borrow_mut().push(join_args(args));
        })?
        .with_name("log")?,
    )?;

    let error_stderr = Rc::clone(&stderr);
    console.set(
        "error",
        Function::new(ctx.clone(), move |args: Rest<Coerced<String>>| {
            error_stderr.borrow_mut().push(join_args(args));
        })?
        .with_name("error")?,
    )?;

    global.set("console", console)?;
    Ok(())
}

fn join_args(args: Rest<Coerced<String>>) -> String {
    args.0
        .into_iter()
        .map(|item| item.0)
        .collect::<Vec<_>>()
        .join(" ")
}

fn sample_by_id(sample_id: &str) -> Option<&'static Sample> {
    SAMPLES.iter().find(|sample| sample.id == sample_id)
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(js_error)
}

fn js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
}
