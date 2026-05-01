use esabi::{
    module::{Exports, ModuleDef},
    Ctx, Result, Value,
};

use crate::runner::{display_value, push_log};

pub(crate) struct ConsoleModule;

impl ModuleDef for ConsoleModule {
    fn declare(decl: &esabi::module::Declarations) -> Result<()> {
        decl.declare("log")?;
        Ok(())
    }

    fn evaluate<'js>(_ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export_copy_fn("log", 1, |args, ctx| {
            let line = args
                .iter()
                .map(|value| display_value(value, ctx))
                .collect::<Vec<_>>()
                .join(" ");
            push_log(line);
            Ok(Value::new_undefined(ctx.clone()))
        })?;
        Ok(())
    }
}
