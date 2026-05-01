use alloc::{rc::Rc, string::ToString as _};
use core::{cell::RefCell, marker::PhantomData, ptr::NonNull};

use boa_engine::{
    object::builtins::JsPromise,
    Context as BoaContext, JsError, JsValue, Source,
};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use boa_engine::context::time::{Clock as BoaClock, JsInstant};

use crate::{
    boa_backend::{
        function::Function,
        job::CompatJobExecutor,
        loader::CompatModuleLoader,
        promise::Promise,
        value::{Object, Value},
    },
    Error, FromJs, Result, Runtime,
};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[derive(Debug, Default)]
struct BrowserClock;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
impl BoaClock for BrowserClock {
    fn now(&self) -> JsInstant {
        let millis = js_sys::Date::now().max(0.0) as u64;
        JsInstant::new(millis / 1000, ((millis % 1000) * 1_000_000) as u32)
    }
}

#[non_exhaustive]
pub struct EvalOptions {
    pub global: bool,
    pub strict: bool,
    pub backtrace_barrier: bool,
    pub promise: bool,
    #[cfg(feature = "std")]
    pub filename: Option<crate::StdString>,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            global: true,
            strict: true,
            backtrace_barrier: false,
            promise: false,
            #[cfg(feature = "std")]
            filename: None,
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub(crate) runtime: Runtime,
    pub(crate) inner: Rc<RefCell<BoaContext>>,
    pub(crate) loader: Rc<CompatModuleLoader>,
    pub(crate) exception_state: Rc<RefCell<Option<JsValue>>>,
}

impl Context {
    pub fn base(runtime: &Runtime) -> Result<Self> {
        Self::full(runtime)
    }

    pub fn custom<I: Intrinsic>(runtime: &Runtime) -> Result<Self> {
        let _ = PhantomData::<I>;
        Self::full(runtime)
    }

    pub fn full(runtime: &Runtime) -> Result<Self> {
        let loader = Rc::new(CompatModuleLoader::new(runtime, runtime.loader_state()));
        let job_executor = CompatJobExecutor::new();
        let builder = BoaContext::builder()
            .module_loader(loader.clone())
            .job_executor(job_executor.clone());
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        let builder = builder.clock(Rc::new(BrowserClock));
        let context = builder
            .build()
            .map_err(|err| Error::new_into_js_message("ContextBuilder", "BoaContext", err.to_string()))?;

        let inner = Rc::new(RefCell::new(context));
        let exception_state = Rc::new(RefCell::new(None));
        runtime.register_context(&inner, &exception_state);

        Ok(Self {
            runtime: runtime.clone(),
            inner,
            loader,
            exception_state,
        })
    }

    pub fn builder() -> ContextBuilder<()> {
        ContextBuilder::default()
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Ctx<'_>) -> R,
    {
        f(Ctx {
            handle: BoaContextHandle::Owned(self.inner.clone()),
            loader: Some(self.loader.clone()),
            runtime: Some(self.runtime.clone()),
            exception_state: self.exception_state.clone(),
            _marker: PhantomData,
        })
    }
}

#[derive(Clone)]
pub(crate) enum BoaContextHandle<'js> {
    Owned(Rc<RefCell<BoaContext>>),
    Borrowed(NonNull<BoaContext>, PhantomData<&'js mut BoaContext>),
}

#[derive(Clone)]
pub struct Ctx<'js> {
    pub(crate) handle: BoaContextHandle<'js>,
    pub(crate) loader: Option<Rc<CompatModuleLoader>>,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) exception_state: Rc<RefCell<Option<JsValue>>>,
    pub(crate) _marker: PhantomData<&'js ()>,
}

impl<'js> core::fmt::Debug for Ctx<'js> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Ctx(..)")
    }
}

impl<'js> Ctx<'js> {
    #[allow(dead_code)]
    pub(crate) fn from_borrowed(boa: &'js mut BoaContext) -> Self {
        Self {
            handle: BoaContextHandle::Borrowed(NonNull::from(boa), PhantomData),
            loader: None,
            runtime: None,
            exception_state: Rc::new(RefCell::new(None)),
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_borrowed_with_state(
        boa: &'js mut BoaContext,
        exception_state: Rc<RefCell<Option<JsValue>>>,
    ) -> Self {
        Self {
            handle: BoaContextHandle::Borrowed(NonNull::from(boa), PhantomData),
            loader: None,
            runtime: None,
            exception_state,
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_loader_borrowed(
        boa: &'js mut BoaContext,
        loader: Rc<CompatModuleLoader>,
        runtime: Option<Runtime>,
    ) -> Self {
        Self {
            handle: BoaContextHandle::Borrowed(NonNull::from(boa), PhantomData),
            loader: Some(loader),
            runtime,
            exception_state: Rc::new(RefCell::new(None)),
            _marker: PhantomData,
        }
    }

    pub(crate) fn with_boa<R>(&self, f: impl FnOnce(&mut BoaContext) -> R) -> R {
        match &self.handle {
            BoaContextHandle::Owned(inner) => {
                let mut boa = inner.borrow_mut();
                f(&mut boa)
            }
            BoaContextHandle::Borrowed(ptr, _) => {
                // The borrowed variant is created from a live `&mut BoaContext` and is only used
                // during that borrow window.
                let boa = unsafe { ptr.as_ptr().as_mut().expect("valid Boa context") };
                f(boa)
            }
        }
    }

    pub(crate) fn module_loader(&self) -> Result<Rc<CompatModuleLoader>> {
        self.loader
            .clone()
            .ok_or_else(|| Error::unsupported("module loader is not available in the current Boa context"))
    }

    pub fn runtime(&self) -> &Runtime {
        self.runtime
            .as_ref()
            .expect("borrowed Boa contexts do not expose Runtime handles")
    }

    pub fn eval<V: FromJs<'js>, S: AsRef<[u8]>>(&self, source: S) -> Result<V> {
        self.eval_with_options(source, EvalOptions::default())
    }

    pub fn eval_with_options<V: FromJs<'js>, S: AsRef<[u8]>>(
        &self,
        source: S,
        options: EvalOptions,
    ) -> Result<V> {
        if options.promise {
            return Err(Error::unsupported(
                "Ctx::eval_with_options does not support promise mode on the Boa backend yet",
            ));
        }

        let source = source.as_ref();
        let value = self.with_boa(|boa| boa.eval(Source::from_bytes(source)));
        let value = value.map_err(|err| self.store_exception(err))?;
        V::from_js(self, Value::from_boa(self.clone(), value))
    }

    pub fn globals(&self) -> Object<'js> {
        let object = self.with_boa(|boa| boa.global_object().clone());
        Object::from_boa_object(self.clone(), object)
    }

    pub fn promise(&self) -> Result<(Promise<'js>, Function<'js>, Function<'js>)> {
        let (promise, resolvers) = self.with_boa(JsPromise::new_pending);
        let promise = Promise::from_js(self, Value::from_boa(self.clone(), promise.into()))?;
        let resolve = Function::from_js(self, Value::from_boa(self.clone(), resolvers.resolve.into()))?;
        let reject = Function::from_js(self, Value::from_boa(self.clone(), resolvers.reject.into()))?;
        Ok((promise, resolve, reject))
    }

    pub fn has_exception(&self) -> bool {
        self.exception_state.borrow().is_some()
    }

    pub fn throw(&self, value: Value<'js>) -> Error {
        *self.exception_state.borrow_mut() = Some(value.into_inner());
        Error::Exception
    }

    pub fn catch(&self) -> Value<'js> {
        let value = self
            .exception_state
            .borrow_mut()
            .take()
            .unwrap_or_else(JsValue::undefined);
        Value::from_boa(self.clone(), value)
    }

    pub(crate) fn store_exception(&self, error: JsError) -> Error {
        let value = self.with_boa(|boa| error.to_opaque(boa));
        *self.exception_state.borrow_mut() = Some(value);
        Error::Exception
    }
}

pub(crate) unsafe fn clone_exception_state(
    ptr: usize,
) -> Rc<RefCell<Option<JsValue>>> {
    let state = unsafe { Rc::from_raw(ptr as *const RefCell<Option<JsValue>>) };
    let cloned = state.clone();
    core::mem::forget(state);
    cloned
}

pub trait Intrinsic {}

impl Intrinsic for () {}

pub mod intrinsic {
    pub struct None;
    pub struct Eval;

    impl super::Intrinsic for None {}
    impl super::Intrinsic for Eval {}
}

pub struct ContextBuilder<I> {
    _marker: PhantomData<I>,
}

impl<I> Default for ContextBuilder<I> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<I> ContextBuilder<I> {
    pub fn with<T>(self) -> ContextBuilder<(I, T)>
    where
        T: Intrinsic,
    {
        ContextBuilder::default()
    }

    pub fn build(self, runtime: &Runtime) -> Result<Context> {
        let _ = self;
        Context::full(runtime)
    }
}
