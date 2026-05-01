use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Result, Value,
};

pub(crate) struct PromisesModule;

impl ModuleDef for PromisesModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("resolvedLabel")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let _ = ctx;
        exports.export_copy_fn("resolvedLabel", 1, |args, ctx| {
            let value = args
                .first()
                .cloned()
                .unwrap_or_else(|| Value::new_int(ctx.clone(), 0))
                .get::<i32>()?;
            let (promise, resolve, _) = ctx.promise()?;
            resolve.call::<_, ()>((Value::new_int(ctx.clone(), value),))?;
            Ok(promise.into_value())
        })?;
        Ok(())
    }
}
