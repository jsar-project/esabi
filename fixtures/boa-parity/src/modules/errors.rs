use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Exception, Function, Result,
};

pub(crate) struct ErrorsModule;

impl ModuleDef for ErrorsModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("failWith")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let fail_with = Function::new(ctx.clone(), |ctx: Ctx<'_>, label: String| -> Result<()> {
            let message = format!("boom:{label}");
            Err(Exception::throw_message(&ctx, &message))
        })?
        .with_name("failWith")?;
        exports.export("failWith", fail_with)?;
        Ok(())
    }
}
