use alloc::{boxed::Box, format, string::{String, ToString as _}};

#[cfg(feature = "std")]
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[cfg(feature = "std")]
use boa_engine::{
    module::{resolve_module_specifier, Module as BoaModule, ModuleLoader as BoaModuleLoader, Referrer},
    Context as BoaContext, JsNativeError, JsResult, JsString,
};

use crate::{
    boa_backend::{module::Declared, value::Object},
    Ctx, Module, Result, Runtime,
};

#[derive(Clone, Debug)]
pub struct ImportAttributes<'js>(Object<'js>);

impl<'js> ImportAttributes<'js> {
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        self.0.get(key)
    }

    pub fn get_type(&self) -> Result<Option<String>> {
        self.get("type")
    }
}

pub trait Resolver {
    fn resolve<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        base: &str,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> Result<String>;
}

pub trait Loader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        name: &str,
        attributes: Option<ImportAttributes<'js>>,
    ) -> Result<Module<'js, Declared>>;
}

#[allow(dead_code)]
#[cfg(feature = "std")]
pub(crate) struct LoaderOpaque {
    pub resolver: Box<dyn Resolver + Send>,
    pub loader: Box<dyn Loader + Send>,
}

#[cfg(feature = "std")]
pub(crate) type SharedLoaderState = Arc<Mutex<Option<LoaderOpaque>>>;

#[cfg(feature = "std")]
pub(crate) struct CompatModuleLoader {
    modules: RefCell<HashMap<String, BoaModule>>,
    state: SharedLoaderState,
    runtime: crate::runtime::WeakRuntime,
}

#[cfg(feature = "std")]
impl CompatModuleLoader {
    pub(crate) fn new(runtime: &Runtime, state: SharedLoaderState) -> Self {
        Self {
            modules: RefCell::new(HashMap::new()),
            state,
            runtime: runtime.weak(),
        }
    }

    pub(crate) fn insert(&self, specifier: impl AsRef<str>, module: BoaModule) -> Option<BoaModule> {
        self.modules
            .borrow_mut()
            .insert(specifier.as_ref().to_string(), module)
    }

    fn lookup(&self, specifier: &str) -> Option<BoaModule> {
        self.modules.borrow().get(specifier).cloned()
    }
}

#[cfg(feature = "std")]
impl Default for CompatModuleLoader {
    fn default() -> Self {
        let runtime = Runtime::new().expect("Boa runtime should be creatable");
        Self::new(&runtime, Arc::new(Mutex::new(None)))
    }
}

#[cfg(feature = "std")]
impl BoaModuleLoader for CompatModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        referrer: Referrer,
        specifier: JsString,
        context: &RefCell<&mut BoaContext>,
    ) -> JsResult<BoaModule> {
        let requested = specifier.to_std_string_escaped();
        let resolved = {
            let mut boa = context.borrow_mut();
            if let Ok(mut state) = self.state.lock() {
                if let Some(state) = state.as_mut() {
                    let runtime = self.runtime.try_ref();
                    let ctx = crate::boa_backend::context::Ctx::from_loader_borrowed(
                        &mut boa,
                        self.clone(),
                        runtime,
                    );
                    state
                        .resolver
                        .resolve(
                            &ctx,
                            &referrer
                                .path()
                                .map(|path| path.to_string_lossy().into_owned())
                                .unwrap_or_default(),
                            requested.as_str(),
                            None,
                        )
                        .unwrap_or_else(|_| requested.clone())
                } else {
                    resolve_module_specifier(None, &specifier, referrer.path(), &mut boa)
                        .map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_else(|_| requested.clone())
                }
            } else {
                requested.clone()
            }
        };

        if let Some(module) = self.lookup(resolved.as_str()).or_else(|| self.lookup(requested.as_str())) {
            return Ok(module);
        }

        let maybe_loaded = {
            let mut boa = context.borrow_mut();
            if let Ok(mut state) = self.state.lock() {
                if let Some(state) = state.as_mut() {
                    let runtime = self.runtime.try_ref();
                    let ctx = crate::boa_backend::context::Ctx::from_loader_borrowed(
                        &mut boa,
                        self.clone(),
                        runtime,
                    );
                    state
                        .loader
                        .load(&ctx, resolved.as_str(), None)
                        .ok()
                        .map(|module| module.as_boa_module())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(module) = maybe_loaded {
            self.insert(resolved, module.clone());
            return Ok(module);
        }

        Err(JsNativeError::typ()
            .with_message(format!("Module could not be found: {}", requested))
            .into())
    }
}
