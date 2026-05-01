use esabi::{module::ModuleDef, Ctx, Function};

pub struct NativeModule;

impl ModuleDef for NativeModule {
    fn declare(declare: &esabi::module::Declarations) -> esabi::Result<()> {
        declare.declare("n")?;
        declare.declare("s")?;
        declare.declare("f")?;
        Ok(())
    }

    fn evaluate<'js>(
        ctx: &Ctx<'js>,
        exports: &esabi::module::Exports<'js>,
    ) -> esabi::Result<()> {
        exports.export("n", 123)?;
        exports.export("s", "abc")?;
        exports.export(
            "f",
            Function::new(ctx.clone(), |a: f64, b: f64| (a + b) * 0.5)?.with_name("f")?,
        )?;
        Ok(())
    }
}
