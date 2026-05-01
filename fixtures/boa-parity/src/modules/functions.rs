use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Function, Object, Result,
};

pub(crate) struct FunctionsModule;

impl ModuleDef for FunctionsModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("upper")?;
        decl.declare("readUrl")?;
        decl.declare("invokeTimer")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let upper = Function::new(ctx.clone(), |input: String| -> Result<String> {
            Ok(input.to_uppercase())
        })?
        .with_name("upper")?;
        exports.export("upper", upper)?;

        let read_url =
            Function::new(ctx.clone(), |ctx: Ctx<'_>, options: Object<'_>| -> Result<String> {
                let fallback = ctx
                    .globals()
                    .get::<_, String>("fallbackUrl")
                    .unwrap_or_else(|_| String::from("https://fallback.example"));
                Ok(options.get::<_, String>("url").unwrap_or(fallback))
            })?
            .with_name("readUrl")?;
        exports.export("readUrl", read_url)?;

        let invoke_timer = Function::new(
            ctx.clone(),
            |ctx: Ctx<'_>, cb: Function<'_>, delay: i32| -> Result<i32> {
                cb.call::<_, ()>((format!("delay={delay}"),))?;
                ctx.globals().set("lastDelay", delay)?;
                Ok(delay)
            },
        )?
        .with_name("invokeTimer")?;
        exports.export("invokeTimer", invoke_timer)?;

        Ok(())
    }
}
