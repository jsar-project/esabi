use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Result, TypedArray,
};

pub(crate) struct TypedArraysModule;

impl ModuleDef for TypedArraysModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("bytes")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let bytes = TypedArray::<u8>::new_copy(ctx.clone(), [1u8, 2, 3, 4])?;
        exports.export("bytes", bytes)?;
        Ok(())
    }
}
