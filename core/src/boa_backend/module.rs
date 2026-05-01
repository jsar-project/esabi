use alloc::{
    ffi::CString,
    format,
    string::{String as StdString, ToString as _},
    vec::Vec,
};
use core::{cell::RefCell, marker::PhantomData};

#[cfg(feature = "std")]
use std::path::Path;

use boa_engine::{
    js_string,
    module::{Module as BoaModule, SyntheticModuleInitializer},
    native_function::NativeFunction,
    object::FunctionObjectBuilder,
    JsString, JsValue,
};

use crate::{
    boa_backend::{
        context::{clone_exception_state, Ctx},
        promise::Promise,
        value::{FromJs, IntoJs, Object, Value},
    },
    Error, Result,
};

pub type ModuleLoadFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_char) -> *mut core::ffi::c_void;

pub trait ModuleDef {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        let _ = decl;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let _ = (ctx, exports);
        Ok(())
    }
}

pub struct Declarations<'js> {
    names: RefCell<Vec<StdString>>,
    _marker: PhantomData<&'js ()>,
}

impl<'js> Declarations<'js> {
    fn new() -> Self {
        Self {
            names: RefCell::new(Vec::new()),
            _marker: PhantomData,
        }
    }

    pub fn declare<N>(&self, name: N) -> Result<&Self>
    where
        N: Into<Vec<u8>>,
    {
        let name = CString::new(name)?;
        self.names.borrow_mut().push(name.to_string_lossy().into_owned());
        Ok(self)
    }

    fn names(&self) -> Vec<StdString> {
        self.names.borrow().clone()
    }
}

pub struct Exports<'js> {
    module: &'js boa_engine::module::SyntheticModule,
    ctx: Ctx<'js>,
}

impl<'js> Exports<'js> {
    fn new(module: &'js boa_engine::module::SyntheticModule, ctx: Ctx<'js>) -> Self {
        Self { module, ctx }
    }

    pub fn export<N, T>(&self, name: N, value: T) -> Result<&Self>
    where
        N: Into<Vec<u8>>,
        T: IntoJs<'js>,
    {
        let name = CString::new(name)?;
        self.export_js(name.to_string_lossy().as_ref(), value)
    }

    fn export_js<T>(&self, name: &str, value: T) -> Result<&Self>
    where
        T: IntoJs<'js>,
    {
        let value = value.into_js(&self.ctx)?;
        self.ctx
            .with_boa(|boa| {
                self.module
                    .set_export(&JsString::from(name), value.clone().into_inner())
                    .map_err(|err| Error::new_into_js_message("module export", "Boa synthetic module", err.to_string()))?;
                let _ = boa;
                Ok::<(), Error>(())
            })?;
        Ok(self)
    }

    pub fn export_copy_fn<N>(
        &self,
        name: N,
        length: usize,
        function: for<'a> fn(&[Value<'a>], &Ctx<'a>) -> Result<Value<'a>>,
    ) -> Result<&Self>
    where
        N: Into<Vec<u8>>,
    {
        let name = CString::new(name)?;
        let export_name = name.to_string_lossy().into_owned();
        let exception_state_ptr = alloc::rc::Rc::as_ptr(&self.ctx.exception_state) as usize;
        let js_value = self.ctx.with_boa(|boa| {
            let native = unsafe { NativeFunction::from_closure(move |_, args, boa_ctx| {
                let call_ctx =
                    Ctx::from_borrowed_with_state(
                        boa_ctx,
                        clone_exception_state(exception_state_ptr),
                    );
                let values = args
                    .iter()
                    .cloned()
                    .map(|arg| Value::from_boa(call_ctx.clone(), arg))
                    .collect::<Vec<_>>();
                match function(&values, &call_ctx) {
                    Ok(value) => Ok(value.into_inner()),
                    Err(Error::Exception) => {
                        Err(boa_engine::JsError::from_opaque(call_ctx.catch().into_inner()))
                    }
                    Err(error) => Err(to_boa_error(error)),
                }
            }) };
            Ok::<JsValue, Error>(
                FunctionObjectBuilder::new(boa.realm(), native)
                    .name(js_string!(export_name.as_str()))
                    .length(length)
                    .constructor(false)
                    .build()
                    .into(),
            )
        })?;

        self.ctx.with_boa(|boa| {
            self.module
                .set_export(&JsString::from(export_name.as_str()), js_value)
                .map_err(|err| Error::new_into_js_message("module export function", "Boa synthetic module", err.to_string()))?;
            let _ = boa;
            Ok::<(), Error>(())
        })?;
        Ok(self)
    }

    pub fn module(&self) -> Ctx<'js> {
        self.ctx.clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Declared;

#[derive(Clone, Copy, Debug)]
pub struct Evaluated;

#[derive(Clone, Debug)]
pub struct Module<'js, T = Declared> {
    inner: BoaModule,
    ctx: Ctx<'js>,
    _type_marker: PhantomData<T>,
}

impl<'js, T> Module<'js, T> {
    fn new(ctx: Ctx<'js>, inner: BoaModule) -> Self {
        Self {
            inner,
            ctx,
            _type_marker: PhantomData,
        }
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    pub(crate) fn as_boa_module(&self) -> BoaModule {
        self.inner.clone()
    }
}

impl<'js> Module<'js, Declared> {
    pub fn declare<N, S>(ctx: Ctx<'js>, name: N, source: S) -> Result<Self>
    where
        N: Into<Vec<u8>>,
        S: Into<Vec<u8>>,
    {
        let name = CString::new(name)?;
        let path = StdString::from(name.to_string_lossy());
        let source = source.into();

        let module = ctx.with_boa(|boa| {
            #[cfg(feature = "std")]
            let source = boa_engine::Source::from_bytes(source.as_slice()).with_path(Path::new(path.as_str()));
            #[cfg(not(feature = "std"))]
            let source = boa_engine::Source::from_bytes(source.as_slice());

            BoaModule::parse(source, None, boa)
                .map_err(|err| Error::new_into_js_message("module source", "Boa module", err.to_string()))
        })?;

        let loader = ctx.module_loader()?;
        loader.insert(path, module.clone());
        Ok(Self::new(ctx, module))
    }

    pub fn declare_def<D, N>(ctx: Ctx<'js>, name: N) -> Result<Self>
    where
        D: ModuleDef,
        N: Into<Vec<u8>>,
    {
        let name = CString::new(name)?;
        let path = StdString::from(name.to_string_lossy());
        let mut declarations = Declarations::new();
        D::declare(&mut declarations)?;
        let export_names = declarations
            .names()
            .into_iter()
            .map(|name| JsString::from(name.as_str()))
            .collect::<Vec<_>>();
        let exception_state_ptr = alloc::rc::Rc::as_ptr(&ctx.exception_state) as usize;

        let module = ctx.with_boa(|boa| {
            let synthetic = BoaModule::synthetic(
                export_names.as_slice(),
                unsafe { SyntheticModuleInitializer::from_closure(move |module, boa_ctx| {
                    let call_ctx =
                        Ctx::from_borrowed_with_state(
                            boa_ctx,
                            clone_exception_state(exception_state_ptr),
                        );
                    let exports = Exports::new(module, call_ctx.clone());
                    match D::evaluate(&call_ctx, &exports) {
                        Ok(()) => Ok(()),
                        Err(Error::Exception) => {
                            Err(boa_engine::JsError::from_opaque(call_ctx.catch().into_inner()))
                        }
                        Err(error) => Err(to_boa_error(error)),
                    }
                }) },
                Some(path.clone().into()),
                None,
                boa,
            );
            Ok::<BoaModule, Error>(synthetic)
        })?;

        let loader = ctx.module_loader()?;
        loader.insert(path, module.clone());
        Ok(Self::new(ctx, module))
    }

    pub fn evaluate<N, S>(ctx: Ctx<'js>, name: N, source: S) -> Result<Promise<'js>>
    where
        N: Into<Vec<u8>>,
        S: Into<Vec<u8>>,
    {
        let module = Self::declare(ctx, name, source)?;
        let (_, promise) = module.eval()?;
        Ok(promise)
    }

    pub fn evaluate_def<D, N>(ctx: Ctx<'js>, name: N) -> Result<(Module<'js, Evaluated>, Promise<'js>)>
    where
        D: ModuleDef,
        N: Into<Vec<u8>>,
    {
        let module = Self::declare_def::<D, N>(ctx, name)?;
        module.eval()
    }

    pub fn eval(self) -> Result<(Module<'js, Evaluated>, Promise<'js>)> {
        self.ctx.with_boa(|boa| {
            let promise = self.inner.load_link_evaluate(boa);
            let promise = Promise::from_js(&self.ctx, Value::from_boa(self.ctx.clone(), promise.into()))?;
            Ok((Module::new(self.ctx.clone(), self.inner), promise))
        })
    }

    pub fn import<S: Into<Vec<u8>>>(
        ctx: &Ctx<'js>,
        specifier: S,
    ) -> Result<(Module<'js, Evaluated>, Promise<'js>)> {
        let specifier = CString::new(specifier)?;
        let script = format!("import * as __esabi_imported from '{}'; __esabi_imported;", specifier.to_string_lossy());
        let module = Self::declare(
            ctx.clone(),
            format!("inline:{}:{}", crate::engine::SELECTED_ENGINE, specifier.to_string_lossy()),
            script.into_bytes(),
        )?;
        module.eval()
    }
}

impl<'js> Module<'js, Evaluated> {
    pub fn namespace(&self) -> Result<Object<'js>> {
        let object = self.ctx.with_boa(|boa| self.inner.namespace(boa));
        Ok(Object::from_boa_object(self.ctx.clone(), object))
    }

    pub fn get<N, T>(&self, name: N) -> Result<T>
    where
        N: AsRef<str>,
        T: FromJs<'js>,
    {
        self.namespace()?.get(name.as_ref())
    }

    pub fn into_declared(self) -> Module<'js, Declared> {
        Module::new(self.ctx, self.inner)
    }
}

pub(crate) fn to_boa_error(error: Error) -> boa_engine::JsError {
    boa_engine::JsNativeError::typ()
        .with_message(error.to_string())
        .into()
}
