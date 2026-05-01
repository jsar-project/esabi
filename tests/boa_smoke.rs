#![cfg(feature = "engine-boa")]

use std::{mem, sync::mpsc, thread};

use esabi::{
    class::{Class, JsClass},
    function::Func,
    loader::{Loader, Resolver},
    module::{Exports, ModuleDef},
    object::{Accessor, Property},
    Context, Constructor, Ctx, Exception, FromJs, Function, Object, Persistent, PromiseState,
    Result, Runtime, Value,
};

#[derive(Clone)]
struct SafeFunctionHandle(Persistent<Function<'static>>);

unsafe impl Send for SafeFunctionHandle {}
unsafe impl Sync for SafeFunctionHandle {}

impl SafeFunctionHandle {
    fn new<'js>(ctx: &Ctx<'js>, function: Function<'js>) -> Self {
        let persistent = Persistent::save(ctx, function);
        let persistent = unsafe { mem::transmute::<Persistent<Function<'js>>, Persistent<Function<'static>>>(persistent) };
        Self(persistent)
    }

    fn restore<'js>(&self, ctx: &Ctx<'js>) -> Result<Function<'js>> {
        self.0.clone().restore(ctx)
    }
}

#[test]
fn boa_runtime_context_eval_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let answer: i32 = ctx.eval("40 + 2")?;
        assert_eq!(answer, 42);

        let globals = ctx.globals();
        globals.set("label", "boa")?;
        globals.set("count", 41)?;

        let object: Object = ctx.eval("({ ok: label === 'boa', next: count + 1 })")?;
        assert!(object.get::<_, bool>("ok")?);
        assert_eq!(object.get::<_, i32>("next")?, 42);

        let value: Value = ctx.eval("({ nested: { answer: 42 } })")?;
        let object = value.into_object().expect("expected object result");
        assert!(object.contains_key("nested")?);

        Ok(())
    })
}

struct MathModule;

impl ModuleDef for MathModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("answer")?;
        decl.declare("add")?;
        Ok(())
    }

    fn evaluate<'js>(_ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("answer", 42)?;
        exports.export_copy_fn("add", 2, |args, ctx| {
            let lhs = args.first().cloned().unwrap_or_else(|| Value::new_int(ctx.clone(), 0));
            let rhs = args.get(1).cloned().unwrap_or_else(|| Value::new_int(ctx.clone(), 0));
            let sum = lhs.get::<i32>()? + rhs.get::<i32>()?;
            Ok(Value::new_int(ctx.clone(), sum))
        })?;
        Ok(())
    }
}

#[test]
fn boa_moduledef_import_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        esabi::Module::declare_def::<MathModule, _>(ctx.clone(), "demo:math")?;
        let module = esabi::Module::declare(
            ctx.clone(),
            "main.mjs",
            br#"
                import { add, answer } from "demo:math";
                export const total = add(8, 10) + answer;
            "#,
        )?;
        let (module, promise) = module.eval()?;

        while runtime.execute_pending_job()? {}

        match promise.state() {
            PromiseState::Pending => panic!("module promise should be settled"),
            PromiseState::Rejected(_) => panic!("module promise should not reject"),
            PromiseState::Fulfilled(_) => {}
        }

        assert_eq!(module.get::<_, i32>("total")?, 60);
        Ok(())
    })
}

#[test]
fn boa_persistent_promise_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let (promise, resolve, _) = ctx.promise()?;
        let promise = Persistent::save(&ctx, promise);
        let resolve = Persistent::save(&ctx, resolve);

        let value = Value::new_int(ctx.clone(), 42);
        resolve.restore(&ctx)?.call::<_, ()>((value,))?;

        while runtime.execute_pending_job()? {}

        match promise.restore(&ctx)?.state() {
            PromiseState::Fulfilled(value) => assert_eq!(value.get::<i32>()?, 42),
            PromiseState::Pending => panic!("promise should be fulfilled"),
            PromiseState::Rejected(_) => panic!("promise should not reject"),
        }

        Ok(())
    })
}

#[test]
fn boa_execute_pending_job_reports_progress() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let (promise, resolve, _) = ctx.promise()?;
        ctx.globals().set("promiseForJobs", promise.clone())?;
        ctx.eval::<(), _>(
            r#"
                globalThis.jobResult = 0;
                promiseForJobs.then((value) => {
                    globalThis.jobResult = value;
                });
            "#,
        )?;
        resolve.call::<_, ()>((Value::new_int(ctx.clone(), 7),))?;

        assert!(runtime.execute_pending_job()?);
        assert!(!runtime.execute_pending_job()?);
        assert_eq!(ctx.globals().get::<_, i32>("jobResult")?, 7);

        match promise.state() {
            PromiseState::Fulfilled(value) => assert_eq!(value.get::<i32>()?, 7),
            PromiseState::Pending => panic!("promise should be fulfilled after one job pass"),
            PromiseState::Rejected(_) => panic!("promise should not reject"),
        }

        Ok(())
    })
}

#[derive(Clone, Copy)]
struct VirtualResolver;

impl Resolver for VirtualResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        _base: &str,
        name: &str,
        _attributes: Option<esabi::loader::ImportAttributes<'js>>,
    ) -> Result<String> {
        Ok(format!("virtual:{name}"))
    }
}

#[derive(Clone, Copy)]
struct VirtualLoader;

impl Loader for VirtualLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attributes: Option<esabi::loader::ImportAttributes<'js>>,
    ) -> Result<esabi::Module<'js, esabi::module::Declared>> {
        let source = match name {
            "virtual:dep" => "export const value = 41;",
            _ => "export const value = 0;",
        };
        esabi::Module::declare(ctx.clone(), name, source)
    }
}

#[test]
fn boa_runtime_set_loader_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.set_loader(VirtualResolver, VirtualLoader);
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let module = esabi::Module::declare(
            ctx.clone(),
            "entry.mjs",
            br#"
                import { value } from "dep";
                export const total = value + 1;
            "#,
        )?;
        let (module, promise) = module.eval()?;

        while runtime.execute_pending_job()? {}

        match promise.state() {
            PromiseState::Fulfilled(_) => {}
            PromiseState::Pending => panic!("loader-backed module promise should settle"),
            PromiseState::Rejected(_) => panic!("loader-backed module promise should not reject"),
        }

        assert_eq!(module.get::<_, i32>("total")?, 42);
        Ok(())
    })
}

#[test]
fn boa_function_new_basic_signatures() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let global = ctx.globals();

        let upper = Function::new(ctx.clone(), |input: String| -> Result<String> {
            Ok(input.to_uppercase())
        })
        .expect("create upper function")
        .with_name("upper")
        .expect("set upper name");
        global.set("upper", upper).expect("register upper");

        let read_url = Function::new(ctx.clone(), |ctx: Ctx<'_>, options: Object<'_>| -> Result<String> {
            let fallback = ctx.globals().get::<_, String>("fallbackUrl").unwrap_or_default();
            Ok(options.get::<_, String>("url").unwrap_or(fallback))
        })
        .expect("create readUrl function");
        global
            .set("fallbackUrl", "https://fallback.example")
            .expect("set fallbackUrl");
        global.set("readUrl", read_url).expect("register readUrl");

        let invoke_timer = Function::new(
            ctx.clone(),
            |ctx: Ctx<'_>, cb: Function<'_>, delay: i32| -> Result<i32> {
                cb.call::<_, ()>((format!("delay={delay}"),))?;
                ctx.globals().set("lastDelay", delay)?;
                Ok(delay)
            },
        )
        .expect("create invokeTimer function");
        global
            .set("invokeTimer", invoke_timer)
            .expect("register invokeTimer");

        let upper_value: String = global
            .get::<_, Function>("upper")
            .expect("read upper from globals")
            .call(("boa",))
            .expect("upper direct call");
        assert_eq!(upper_value, "BOA");

        let options = Object::new(ctx.clone()).expect("create options object");
        let read_url_value: String = global
            .get::<_, Function>("readUrl")
            .expect("read readUrl from globals")
            .call((options,))
            .expect("readUrl direct call");
        assert_eq!(read_url_value, "https://fallback.example");

        global.set("timerMessage", "").expect("set timerMessage");
        let timer_cb: Function = ctx
            .eval(
            r#"(message) => {
                globalThis.timerMessage = message;
            }"#,
        )
            .expect("build timer callback");
        let invoked_delay: i32 = global
            .get::<_, Function>("invokeTimer")
            .expect("read invokeTimer from globals")
            .call((timer_cb, 12))
            .expect("invokeTimer direct call");
        assert_eq!(invoked_delay, 12);

        assert_eq!(
            global
                .get::<_, String>("timerMessage")
                .expect("read timerMessage"),
            "delay=12"
        );
        assert_eq!(
            global.get::<_, i32>("lastDelay").expect("read lastDelay"),
            12
        );

        Ok(())
    })
}

#[test]
fn boa_exception_and_func_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let globals = ctx.globals();
        globals.set("cat", Func::from(|a: String, b: String| format!("{a}{b}")))?;

        let concatenated: String = ctx.eval(r#"cat("boa", "-ok")"#)?;
        assert_eq!(concatenated, "boa-ok");

        let fail = Function::new(ctx.clone(), |ctx: Ctx<'_>| -> Result<()> {
            Err(Exception::throw_message(&ctx, "boom"))
        })?;
        globals.set("fail", fail)?;

        let error = ctx.eval::<(), _>("fail()").expect_err("fail() should throw");
        assert!(matches!(error, esabi::Error::Exception));

        let exception = Exception::from_js(&ctx, ctx.catch())?;
        assert_eq!(exception.message().as_deref(), Some("boom"));

        Ok(())
    })
}

#[derive(Debug)]
struct Greeter {
    label: String,
}

fn new_greeter<'js>(ctx: Ctx<'js>, label: String) -> Result<Value<'js>> {
    let greeter = Class::instance(ctx, Greeter { label })?;
    Ok(greeter.into_object().into_value())
}

impl<'js> JsClass<'js> for Greeter {
    const NAME: &'static str = "Greeter";
    type Mutable = esabi::class::Writable;

    fn init_prototype(_ctx: &Ctx<'js>, prototype: &Object<'js>) -> Result<()> {
        prototype.set(
            "greet",
            Func::from(|this: esabi::function::This<Class<'_, Greeter>>| -> Result<String> {
                Ok(format!("hello {}", this.0.borrow().label))
            }),
        )?;
        prototype.prop(
            "label",
            Accessor::new(
                Func::from(
                    |this: esabi::function::This<Class<'_, Greeter>>| -> Result<String> {
                        Ok(this.0.borrow().label.clone())
                    },
                ),
                Func::from(
                    |this: esabi::function::This<Class<'_, Greeter>>,
                     label: String|
                     -> Result<()> {
                        this.0.borrow_mut().label = label;
                        Ok(())
                    },
                ),
            )
            .enumerable(),
        )?;
        prototype.prop(
            "kind",
            Property::from("greeter")
                .configurable()
                .enumerable()
                .writable(),
        )?;
        Ok(())
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(ctx.clone(), new_greeter)?))
    }
}

#[test]
fn boa_class_constructor_and_methods_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        Class::<Greeter>::define(&ctx.globals()).unwrap_or_else(|error| {
            let message = Exception::from_js(&ctx, ctx.catch())
                .ok()
                .and_then(|exception| exception.message())
                .unwrap_or_else(|| "<missing message>".to_string());
            panic!("define failed: {error:?}: {message}");
        });
        let direct = Class::instance(
            ctx.clone(),
            Greeter {
                label: "direct".into(),
            },
        )?;
        ctx.globals().set("directGreeter", direct.into_object())?;

        let value: String = ctx
            .eval(
            r#"
                (() => {
                    const constructed = new Greeter("boa");
                    directGreeter.label = "friend";
                    if (directGreeter.greet() !== "hello friend") {
                        return "direct-label";
                    }
                    if (directGreeter.label !== "friend") {
                        return "label-getter";
                    }
                    if (constructed.greet() !== "hello boa") {
                        return "constructed-greet";
                    }
                    const kindDesc = Object.getOwnPropertyDescriptor(Greeter.prototype, "kind");
                    if (!kindDesc || kindDesc.enumerable !== true) {
                        return "prototype-prop";
                    }
                    return `${directGreeter.greet()}|${directGreeter.label}|${constructed.greet()}|${kindDesc.enumerable}`;
                })()
            "#,
        )
            .unwrap_or_else(|error| {
                let message = Exception::from_js(&ctx, ctx.catch())
                    .ok()
                    .and_then(|exception| exception.message())
                    .unwrap_or_else(|| "<missing message>".to_string());
                panic!("eval failed: {error:?}: {message}");
            });
        assert_eq!(value, "hello friend|friend|hello boa|true");
        Ok(())
    })
}

#[test]
fn boa_threadsafe_dispatch_pattern_smoke() -> Result<()> {
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    context.with(|ctx| {
        let globals = ctx.globals();
        globals.set("threadsafeMessage", "")?;

        let callback: Function = ctx.eval(
            r#"(message) => {
                globalThis.threadsafeMessage = message;
            }"#,
        )?;
        let callback = SafeFunctionHandle::new(&ctx, callback);

        let (tx, rx) = mpsc::channel::<String>();
        thread::spawn(move || {
            tx.send("from-thread".to_string()).expect("send payload");
        })
        .join()
        .expect("join sender thread");

        let payload = rx.recv().expect("receive payload");
        callback.restore(&ctx)?.call::<_, ()>((payload,))?;
        assert_eq!(globals.get::<_, String>("threadsafeMessage")?, "from-thread");

        let failing: Function = ctx.eval(
            r#"() => {
                throw new Error("queued boom");
            }"#,
        )?;
        let failing = SafeFunctionHandle::new(&ctx, failing);
        let error = failing
            .restore(&ctx)?
            .call::<_, ()>(())
            .expect_err("queued callback should throw");
        assert!(matches!(error, esabi::Error::Exception));

        let exception = Exception::from_js(&ctx, ctx.catch())?;
        assert_eq!(exception.message().as_deref(), Some("queued boom"));

        Ok(())
    })
}
