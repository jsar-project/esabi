#![allow(dead_code)]

use esabi::{
    class::{JsClass, Readable, Trace, Tracer},
    Class, Context, JsLifetime, Null, Runtime,
};
use std::sync::Mutex;

static VALIDATE: Mutex<(bool, bool, bool)> = Mutex::new((false, false, false));

pub struct A;

impl<'js> Trace<'js> for A {
    fn trace(&self, _tracer: Tracer<'_, 'js>) {
        VALIDATE.lock().unwrap().0 = true;
    }
}

pub struct B;

impl<'js> Trace<'js> for B {
    fn trace(&self, _tracer: Tracer<'_, 'js>) {
        VALIDATE.lock().unwrap().1 = true;
    }
}

pub struct C;

impl<'js> Trace<'js> for C {
    fn trace(&self, _tracer: Tracer<'_, 'js>) {
        VALIDATE.lock().unwrap().2 = true;
    }
}

#[derive(Trace, JsLifetime)]
pub struct TraceStruct {
    a: A,
    #[qjs(skip_trace)]
    b: B,
    c: C,
}

impl<'js> JsClass<'js> for TraceStruct {
    const NAME: &'static str = "TraceStruct";

    type Mutable = Readable;

    fn prototype(_ctx: &esabi::Ctx<'js>) -> esabi::Result<Option<esabi::Object<'js>>> {
        Ok(None)
    }

    fn constructor(
        _ctx: &esabi::Ctx<'js>,
    ) -> esabi::Result<Option<esabi::function::Constructor<'js>>> {
        Ok(None)
    }
}

#[derive(Trace, JsLifetime)]
pub enum TraceEnum {
    A(A),
    B(#[qjs(skip_trace)] B),
    C,
}

#[derive(Trace, JsLifetime)]
#[allow(dead_code)]
pub struct TraceBuiltins<'js> {
    constructor: esabi::Constructor<'js>,
    promise: esabi::Promise<'js>,
    proxy: esabi::Proxy<'js>,
    array_buffer: esabi::ArrayBuffer<'js>,
    typed_array: esabi::TypedArray<'js, u8>,
}

impl<'js> JsClass<'js> for TraceEnum {
    const NAME: &'static str = "TraceEnum";

    type Mutable = Readable;

    fn prototype(_ctx: &esabi::Ctx<'js>) -> esabi::Result<Option<esabi::Object<'js>>> {
        Ok(None)
    }

    fn constructor(
        _ctx: &esabi::Ctx<'js>,
    ) -> esabi::Result<Option<esabi::function::Constructor<'js>>> {
        Ok(None)
    }
}

pub fn main() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        let cls = Class::instance(ctx.clone(), TraceStruct { a: A, b: B, c: C }).unwrap();
        ctx.globals().set("t", cls).unwrap();
    });

    rt.run_gc();

    assert_eq!(*VALIDATE.lock().unwrap(), (true, false, true));

    ctx.with(|ctx| {
        ctx.globals().set("t", Null).unwrap();
    });

    rt.run_gc();

    *VALIDATE.lock().unwrap() = (false, false, false);

    ctx.with(|ctx| {
        let cls = Class::instance(ctx.clone(), TraceEnum::A(A)).unwrap();
        ctx.globals().set("t", cls).unwrap();
    });

    rt.run_gc();
    assert_eq!(*VALIDATE.lock().unwrap(), (true, false, false));
}
