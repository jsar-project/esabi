use esabi::{
    class::{Class, JsClass},
    function::{Constructor, This},
    module::{Exports, ModuleDef},
    object::{Accessor, Property},
    Ctx, Func, Function, Object, Result,
};

pub(crate) struct ClassBasicModule;

#[derive(Debug)]
struct Greeter {
    label: String,
}

fn new_greeter<'js>(ctx: Ctx<'js>, label: String) -> Result<esabi::Value<'js>> {
    let greeter = Class::instance(ctx, Greeter { label })?;
    Ok(greeter.into_object().into_value())
}

impl<'js> JsClass<'js> for Greeter {
    const NAME: &'static str = "Greeter";
    type Mutable = esabi::class::Writable;

    fn init_prototype(_ctx: &Ctx<'js>, prototype: &Object<'js>) -> Result<()> {
        prototype.set(
            "greet",
            Func::from(|this: This<Class<'_, Greeter>>| -> Result<String> {
                Ok(format!("hello {}", this.0.borrow().label))
            }),
        )?;
        prototype.prop(
            "label",
            Accessor::new(
                Func::from(|this: This<Class<'_, Greeter>>| -> Result<String> {
                    Ok(this.0.borrow().label.clone())
                }),
                Func::from(|this: This<Class<'_, Greeter>>, label: String| -> Result<()> {
                    this.0.borrow_mut().label = label;
                    Ok(())
                }),
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

impl ModuleDef for ClassBasicModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("Greeter")?;
        decl.declare("makeGreeter")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let constructor = Class::<Greeter>::create_constructor(ctx)?
            .expect("Greeter constructor should exist");
        exports.export("Greeter", constructor.clone())?;

        let make_greeter = Function::new(ctx.clone(), new_greeter)?
        .with_name("makeGreeter")?;
        exports.export("makeGreeter", make_greeter)?;
        Ok(())
    }
}
