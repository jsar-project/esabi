#![cfg(all(feature = "engine-boa", feature = "macro"))]

use std::sync::atomic::{AtomicU32, Ordering};

use esabi::{
    atom::PredefinedAtom, class::Trace, Class, Context, Ctx, JsLifetime, Object, Result, Runtime,
};

#[derive(Clone, Trace, JsLifetime)]
#[esabi::class]
struct Counter {
    #[qjs(get, set)]
    value: i32,
}

static DEFAULT_GAUGE_VALUE: AtomicU32 = AtomicU32::new(42);

#[derive(Clone, Trace, JsLifetime)]
#[esabi::class]
struct Gauge {
    current: u32,
}

#[derive(Clone, Trace, JsLifetime)]
#[esabi::class(rename_all = "camelCase")]
struct CamelBox {
    #[qjs(get, set)]
    some_value: u32,
    #[qjs(get, set, enumerable)]
    another_value: u32,
}

#[derive(Clone, Trace, JsLifetime)]
#[esabi::class]
struct RichMethods {
    value: u32,
    another_value: u32,
}

#[derive(Clone, Trace, JsLifetime)]
#[esabi::class]
struct SymbolMethods {
    value: u32,
}

#[esabi::methods]
impl Counter {
    #[qjs(constructor)]
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn inc(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}

#[esabi::methods]
impl Gauge {
    #[qjs(constructor)]
    pub fn new(value: u32) -> Self {
        Self { current: value }
    }

    #[qjs(get, rename = "value")]
    pub fn get_value(&self) -> u32 {
        self.current
    }

    #[qjs(set, rename = "value")]
    pub fn set_value(&mut self, value: u32) {
        self.current = value;
    }

    #[qjs(static)]
    pub fn is_default_value(value: u32) -> bool {
        value == DEFAULT_GAUGE_VALUE.load(Ordering::SeqCst)
    }

    #[qjs(static, get, rename = "defaultValue")]
    pub fn default_value() -> u32 {
        DEFAULT_GAUGE_VALUE.load(Ordering::SeqCst)
    }

    #[qjs(static, set, rename = "defaultValue")]
    pub fn set_default_value(value: u32) {
        DEFAULT_GAUGE_VALUE.store(value, Ordering::SeqCst);
    }

    #[qjs(prop, rename = "kind", configurable, enumerable, writable)]
    pub fn kind() -> &'static str {
        "gauge"
    }
}

#[esabi::methods(rename_all = "camelCase")]
impl RichMethods {
    #[qjs(constructor)]
    pub fn new(value: u32) -> Self {
        Self {
            value,
            another_value: value,
        }
    }

    #[qjs(get, rename = "value")]
    pub fn get_value(&self) -> u32 {
        self.value
    }

    #[qjs(set, rename = "value")]
    pub fn set_value(&mut self, value: u32) {
        self.value = value;
    }

    #[qjs(get, rename = "anotherValue", enumerable)]
    pub fn get_another_value(&self) -> u32 {
        self.another_value
    }

    #[qjs(set, rename = "anotherValue", enumerable)]
    pub fn set_another_value(&mut self, value: u32) {
        self.another_value = value;
    }

    #[qjs(static)]
    pub fn compare(a: &Self, b: &Self) -> bool {
        a.value == b.value && a.another_value == b.another_value
    }

    #[qjs(static, get, rename = "defaultValue")]
    pub fn default_value() -> u32 {
        42
    }

    #[qjs(static, get, rename = "staticPair")]
    pub fn get_static_pair() -> u32 {
        7
    }

    #[qjs(static, set, rename = "staticPair")]
    pub fn set_static_pair(_value: u32) {}

    #[qjs(skip)]
    pub fn inner_function(&self) {}

    #[qjs(prop, rename = "kind", configurable, enumerable, writable)]
    pub fn kind() -> &'static str {
        "rich"
    }

    pub fn make_done_object<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let result = Object::new(ctx)?;
        result.set(
            "nextValue",
            esabi::Func::from(|| true),
        )?;
        Ok(result)
    }
}

#[esabi::methods]
impl SymbolMethods {
    #[qjs(constructor)]
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    #[qjs(static, rename = PredefinedAtom::SymbolHasInstance)]
    pub fn has_instance<'js>(_value: esabi::Value<'js>) -> bool {
        false
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        "SymbolMethods"
    }

    #[qjs(rename = PredefinedAtom::SymbolIterator)]
    pub fn iterate<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let result = Object::new(ctx)?;
        result.set(
            PredefinedAtom::Next,
            esabi::Func::from(|ctx: Ctx<'js>| -> Result<Object<'js>> {
                let result = Object::new(ctx)?;
                result.set(PredefinedAtom::Done, true)?;
                Ok(result)
            }),
        )?;
        Ok(result)
    }
}

#[test]
fn boa_class_and_methods_macro_smoke() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        Class::<Counter>::define(&ctx.globals()).unwrap();
        let existing = Class::instance(ctx.clone(), Counter { value: 10 }).unwrap();
        ctx.globals().set("existing", existing.clone()).unwrap();

        ctx.eval::<(), _>(
            r#"
            const a = new Counter(3);
            if (a.value !== 3) {
                throw new Error("constructor/get failed");
            }
            a.value = 7;
            if (a.value !== 7) {
                throw new Error("setter failed");
            }
            if (a.inc() !== 8) {
                throw new Error("method failed");
            }
            if (existing.inc() !== 11) {
                throw new Error("existing instance failed");
            }
        "#,
        )
        .unwrap();

        assert_eq!(existing.borrow().value, 11);
    });
}

#[test]
fn boa_class_methods_static_and_props_macro_smoke() {
    DEFAULT_GAUGE_VALUE.store(42, Ordering::SeqCst);

    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        Class::<Gauge>::define(&ctx.globals()).unwrap();
        ctx.eval::<(), _>("const gauge = new Gauge(5);").unwrap();

        ctx.eval::<(), _>(
            r#"
            if (gauge.value !== 5) {
                throw new Error("instance getter failed");
            }
            gauge.value = 8;
            if (gauge.value !== 8) {
                throw new Error("instance setter failed");
            }
        "#,
        )
        .unwrap();

        ctx.eval::<(), _>(
            r#"
            if (Gauge.defaultValue !== 42) {
                throw new Error("static getter failed");
            }
        "#,
        )
        .unwrap();

        ctx.eval::<(), _>(
            r#"
            Gauge.defaultValue = 9;
            if (Gauge.defaultValue !== 9) {
                throw new Error("static setter failed");
            }
        "#,
        )
        .unwrap();

        ctx.eval::<(), _>(
            r#"
            if (!Gauge.is_default_value(9)) {
                throw new Error("static method failed");
            }
        "#,
        )
        .unwrap();

        ctx.eval::<(), _>(
            r#"
            const defaultDesc = Object.getOwnPropertyDescriptor(Gauge, "defaultValue");
            if (!defaultDesc || typeof defaultDesc.get !== "function" || typeof defaultDesc.set !== "function") {
                throw new Error("static accessor descriptor failed");
            }
        "#,
        )
        .unwrap();

        ctx.eval::<(), _>(
            r#"
            const kindDesc = Object.getOwnPropertyDescriptor(Gauge.prototype, "kind");
            if (!kindDesc || kindDesc.value !== "gauge") {
                throw new Error("data property value failed");
            }
            if (kindDesc.configurable !== true || kindDesc.enumerable !== true || kindDesc.writable !== true) {
                throw new Error("data property descriptor failed");
            }
        "#,
        )
        .unwrap();
    });

    assert_eq!(DEFAULT_GAUGE_VALUE.load(Ordering::SeqCst), 9);
}

#[test]
fn boa_class_fields_rename_all_macro_smoke() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        let cls = Class::instance(
            ctx.clone(),
            CamelBox {
                some_value: 1,
                another_value: 2,
            },
        )
        .unwrap();
        ctx.globals().set("boxValue", cls.clone()).unwrap();

        ctx.eval::<(), _>(
            r#"
            if (boxValue.someValue !== 1) {
                throw new Error("camelCase getter failed");
            }
            if (boxValue.anotherValue !== 2) {
                throw new Error("enumerable getter failed");
            }
            boxValue.someValue = 3;
            if (boxValue.someValue !== 3) {
                throw new Error("camelCase setter failed");
            }
            const proto = Object.getPrototypeOf(boxValue);
            if (!Object.keys(proto).includes("anotherValue")) {
                throw new Error("enumerable accessor missing");
            }
            if (Object.keys(proto).includes("someValue")) {
                throw new Error("non-enumerable accessor leaked");
            }
        "#,
        )
        .unwrap();

        let borrowed = cls.borrow();
        assert_eq!(borrowed.some_value, 3);
        assert_eq!(borrowed.another_value, 2);
    });
}

#[test]
fn boa_class_methods_extended_macro_smoke() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        Class::<RichMethods>::define(&ctx.globals()).unwrap();
        ctx.globals()
            .set(
                "existingRich",
                RichMethods {
                    value: 1,
                    another_value: 2,
                },
            )
            .unwrap();

        ctx.eval::<(), _>(
            r#"
            if (existingRich.value !== 1) {
                throw new Error("instance getter failed");
            }
            if (existingRich.anotherValue !== 2) {
                throw new Error("enumerable getter failed");
            }
            existingRich.value = 5;
            if (existingRich.value !== 5) {
                throw new Error("instance setter failed");
            }
            const nv = new RichMethods(5);
            if (nv.value !== 5) {
                throw new Error("constructor failed");
            }
            existingRich.anotherValue = 5;
            if (!RichMethods.compare(existingRich, nv)) {
                throw new Error("static compare failed");
            }
            if (nv.inner_function !== undefined) {
                throw new Error("skip did not skip");
            }
            if (RichMethods.defaultValue !== 42) {
                throw new Error("static getter failed");
            }
            if (RichMethods.staticPair !== 7) {
                throw new Error("static accessor getter failed");
            }
            RichMethods.staticPair = 11;
            const proto = RichMethods.prototype;
            const kindDesc = Object.getOwnPropertyDescriptor(proto, "kind");
            if (!kindDesc || kindDesc.value !== "rich") {
                throw new Error("string prop value failed");
            }
            if (kindDesc.configurable !== true || kindDesc.enumerable !== true || kindDesc.writable !== true) {
                throw new Error("string prop descriptor failed");
            }
            const doneObject = nv.makeDoneObject();
            if (typeof doneObject.nextValue !== "function" || doneObject.nextValue() !== true) {
                throw new Error("ctx/object return method failed");
            }
            if (!Object.keys(proto).includes("anotherValue")) {
                throw new Error("enumerable accessor descriptor missing");
            }
            if (Object.keys(proto).includes("value")) {
                throw new Error("non-enumerable accessor descriptor leaked");
            }
        "#,
        )
        .unwrap();
    });
}

#[test]
fn boa_class_symbol_named_members_macro_smoke() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        Class::<SymbolMethods>::define(&ctx.globals()).unwrap();

        ctx.eval::<(), _>(
            r#"
            const instance = new SymbolMethods(1);
            if (typeof SymbolMethods[Symbol.hasInstance] !== "function") {
                throw new Error("static Symbol.hasInstance not attached");
            }
            if (SymbolMethods[Symbol.hasInstance]({}) !== false) {
                throw new Error("static Symbol.hasInstance returned wrong value");
            }
            const proto = SymbolMethods.prototype;
            const tagDesc = Object.getOwnPropertyDescriptor(proto, Symbol.toStringTag);
            if (!tagDesc || tagDesc.value !== "SymbolMethods") {
                throw new Error("Symbol.toStringTag value failed");
            }
            if (tagDesc.get !== undefined || tagDesc.set !== undefined) {
                throw new Error("Symbol.toStringTag descriptor shape failed");
            }
            if (tagDesc.configurable !== true || tagDesc.enumerable !== false || tagDesc.writable !== false) {
                throw new Error("Symbol.toStringTag descriptor flags failed");
            }
            const fake = Object.create(proto);
            if (Object.prototype.toString.call(fake) !== "[object SymbolMethods]") {
                throw new Error("Symbol.toStringTag lookup failed");
            }
            const iterator = instance[Symbol.iterator]();
            if (typeof iterator.next !== "function") {
                throw new Error("Symbol.iterator result missing next");
            }
            const step = iterator.next();
            if (step.done !== true) {
                throw new Error("iterator next result failed");
            }
            for (const _value of instance) {
                throw new Error("iterator should finish immediately");
            }
        "#,
        )
        .unwrap();
    });
}
