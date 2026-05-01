use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Result, Value,
};

pub(crate) struct MathModule;

impl ModuleDef for MathModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("answer")?;
        decl.declare("add")?;
        Ok(())
    }

    fn evaluate<'js>(_ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("answer", 42)?;
        exports.export_copy_fn("add", 2, |args, ctx| {
            let lhs = args
                .first()
                .cloned()
                .unwrap_or_else(|| Value::new_int(ctx.clone(), 0));
            let rhs = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| Value::new_int(ctx.clone(), 0));
            Ok(Value::new_int(
                ctx.clone(),
                lhs.get::<i32>()? + rhs.get::<i32>()?,
            ))
        })?;
        Ok(())
    }
}
