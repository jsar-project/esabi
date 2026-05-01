use std::{cell::RefCell, fmt::Write as _};

use esabi::{Context, Ctx, Module, PromiseState, Result, Runtime, Value};
use serde::Serialize;

use crate::{fixture_data::BoaFixture, modules::register_fixture_modules};

thread_local! {
    static LOGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RunOutput {
    pub ok: bool,
    pub result: Option<String>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

pub fn run_fixture(fixture: &BoaFixture) -> RunOutput {
    run_source(fixture.code)
}

pub fn run_source(source: &str) -> RunOutput {
    clear_logs();

    let runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => return failure_output(error.to_string()),
    };
    let context = match Context::full(&runtime) {
        Ok(context) => context,
        Err(error) => return failure_output(error.to_string()),
    };

    let output = context.with(|ctx| run_in_context(&runtime, &ctx, source));
    match output {
        Ok(output) => output,
        Err(error) => failure_output(error.to_string()),
    }
}

pub(crate) fn push_log(line: String) {
    LOGS.with(|logs| logs.borrow_mut().push(line));
}

pub(crate) fn display_value<'js>(value: &Value<'js>, ctx: &Ctx<'js>) -> String {
    if value.is_undefined() {
        return String::from("undefined");
    }
    if value.is_null() {
        return String::from("null");
    }
    if let Some(boolean) = value.as_bool() {
        return boolean.to_string();
    }
    if let Some(integer) = value.as_int() {
        return integer.to_string();
    }
    if let Some(number) = value.as_float() {
        return number.to_string();
    }
    if value.is_string() {
        if let Ok(string) = value.clone().get::<String>() {
            return string;
        }
    }

    let mut fallback = String::new();
    let _ = write!(&mut fallback, "{value:?}");
    if fallback.is_empty() {
        let _ = ctx;
        String::from("<unprintable>")
    } else {
        fallback
    }
}

fn clear_logs() {
    LOGS.with(|logs| logs.borrow_mut().clear());
}

fn cloned_logs() -> Vec<String> {
    LOGS.with(|logs| logs.borrow().clone())
}

fn run_in_context<'js>(runtime: &Runtime, ctx: &Ctx<'js>, source: &str) -> Result<RunOutput> {
    register_fixture_modules(ctx)?;

    let module = Module::declare(ctx.clone(), "playground.mjs", source.as_bytes())?;
    let (module, promise) = module.eval()?;
    while runtime.execute_pending_job()? {}

    match promise.state() {
        PromiseState::Pending => {
            return Ok(RunOutput {
                ok: false,
                result: None,
                logs: cloned_logs(),
                error: Some(String::from("module promise is still pending")),
            });
        }
        PromiseState::Rejected(value) => {
            return Ok(RunOutput {
                ok: false,
                result: None,
                logs: cloned_logs(),
                error: Some(stringify_value(ctx, &value)),
            });
        }
        PromiseState::Fulfilled(_) => {}
    }

    let namespace = module.namespace()?;
    let result = if namespace.contains_key("result")? {
        let value = namespace.get::<_, Value>("result")?;
        Some(display_value(&value, ctx))
    } else {
        None
    };

    Ok(RunOutput {
        ok: true,
        result,
        logs: cloned_logs(),
        error: None,
    })
}

fn stringify_value<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> String {
    if value.is_string() {
        if let Ok(string) = value.clone().get::<String>() {
            return string;
        }
    }

    let globals = ctx.globals();
    if globals.set("__boaFixtureValue", value.clone()).is_ok() {
        if let Ok(stringified) = ctx.eval::<String, _>("String(globalThis.__boaFixtureValue)") {
            return stringified;
        }
    }

    display_value(value, ctx)
}

fn failure_output(error: String) -> RunOutput {
    RunOutput {
        ok: false,
        result: None,
        logs: cloned_logs(),
        error: Some(error),
    }
}
