use esabi::atom::PredefinedAtom;
use esabi::class::Trace;
use esabi::{Ctx, JsLifetime, Null, Object, Result, Value};

#[derive(Clone, Trace, JsLifetime)]
#[esabi::class]
pub struct MyClass {
    #[qjs(skip_trace)]
    data: String,
}

#[esabi::methods(rename_all = "camelCase")]
impl MyClass {
    #[qjs(constructor)]
    pub fn new(data: String) -> Self {
        Self { data }
    }

    #[qjs(get)]
    fn data(&self) -> String {
        self.data.clone()
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let obj = Object::new(ctx)?;
        obj.set("data", &self.data)?;
        Ok(obj.into_value())
    }

    #[allow(clippy::inherent_to_string)]
    #[qjs(rename = PredefinedAtom::ToString)]
    fn to_string(&self) -> String {
        format!("MyClass({})", self.data)
    }

    #[qjs(rename = PredefinedAtom::SymbolToPrimitive)]
    fn to_primitive<'js>(&self, ctx: Ctx<'js>, hint: String) -> Result<Value<'js>> {
        if hint == "string" {
            return Ok(esabi::String::from_str(ctx, &self.data)?.into_value());
        }
        Ok(Null.into_value(ctx))
    }
}
