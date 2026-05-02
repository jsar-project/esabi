use rquickjs::{
    loader::{ImportAttributes, Loader, Resolver},
    module::Declared,
    Context, Ctx, Module, Result, Runtime,
};

struct IdentityResolver;

impl Resolver for IdentityResolver {
    fn resolve<'js>(
        &mut self,
        _ctx: &Ctx<'js>,
        _base: &str,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<String> {
        Ok(name.into())
    }
}

struct StaticLoader;

impl Loader for StaticLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        _attributes: Option<ImportAttributes<'js>>,
    ) -> Result<Module<'js, Declared>> {
        match name {
            "dep" => Module::declare(ctx.clone(), name, "export const value = 41;"),
            _ => unreachable!("unexpected module request: {name}"),
        }
    }
}

fn main() {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        ctx.with(|ctx| {
            let value: i32 = ctx.eval("40 + 2").unwrap();
            assert_eq!(value, 42);
        });

        rt.set_loader(IdentityResolver, StaticLoader);
        ctx.with(|ctx| {
            Module::evaluate(
                ctx.clone(),
                "entry",
                r#"
                    import { value } from "dep";
                    globalThis.answer = value + 1;
                "#,
            )
            .unwrap()
            .finish::<()>()
            .unwrap();

            let answer: i32 = ctx.globals().get("answer").unwrap();
            assert_eq!(answer, 42);
        });
    }
}
