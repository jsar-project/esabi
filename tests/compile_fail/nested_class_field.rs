use esabi::{class::Trace, JsLifetime};

#[derive(Trace, JsLifetime, Default, Clone)]
#[esabi::class]
struct Outer {
    #[qjs(get, set)]
    inner: Inner,
}

#[derive(Trace, JsLifetime, Default, Clone)]
#[esabi::class]
struct Inner {
    #[qjs(get, set)]
    value: String,
}

fn main() {}
