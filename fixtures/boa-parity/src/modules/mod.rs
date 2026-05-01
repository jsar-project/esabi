mod class_basic;
mod console;
mod errors;
mod functions;
mod math;
mod objects;
mod promises;
mod typed_arrays;

use esabi::{Ctx, Module, Result};

pub(crate) fn register_fixture_modules<'js>(ctx: &Ctx<'js>) -> Result<()> {
    Module::declare_def::<math::MathModule, _>(ctx.clone(), "demo:math")?;
    Module::declare_def::<console::ConsoleModule, _>(ctx.clone(), "demo:console")?;
    Module::declare_def::<functions::FunctionsModule, _>(ctx.clone(), "demo:functions")?;
    Module::declare_def::<objects::ObjectsModule, _>(ctx.clone(), "demo:objects")?;
    Module::declare_def::<errors::ErrorsModule, _>(ctx.clone(), "demo:errors")?;
    Module::declare_def::<promises::PromisesModule, _>(ctx.clone(), "demo:promises")?;
    Module::declare_def::<class_basic::ClassBasicModule, _>(ctx.clone(), "demo:class-basic")?;
    Module::declare_def::<typed_arrays::TypedArraysModule, _>(ctx.clone(), "demo:typed-arrays")?;
    Ok(())
}
