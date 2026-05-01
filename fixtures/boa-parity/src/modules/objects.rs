use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Object, Result, Value,
};

pub(crate) struct ObjectsModule;

impl ModuleDef for ObjectsModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("fixtureBox")?;
        decl.declare("makePoint")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let fixture_box = Object::new(ctx.clone())?;
        fixture_box.set("label", "boa-box")?;
        let nested = Object::new(ctx.clone())?;
        nested.set("ok", true)?;
        fixture_box.set("nested", nested)?;
        exports.export("fixtureBox", fixture_box)?;

        exports.export_copy_fn("makePoint", 2, |args, ctx| {
            let x = args
                .first()
                .cloned()
                .unwrap_or_else(|| Value::new_int(ctx.clone(), 0))
                .get::<i32>()?;
            let y = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| Value::new_int(ctx.clone(), 0))
                .get::<i32>()?;
            let point = Object::new(ctx.clone())?;
            point.set("x", x)?;
            point.set("y", y)?;
            point.set("sum", x + y)?;
            Ok(point.into_value())
        })?;

        Ok(())
    }
}
