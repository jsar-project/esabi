use alloc::{rc, sync::{Arc, Weak}, vec::Vec};
#[cfg(feature = "loader")]
use alloc::boxed::Box;

use core::cell::RefCell;

use boa_engine::Context as BoaContext;

use crate::{
    boa_backend::job::CompatJobExecutor,
    boa_backend::loader::SharedLoaderState,
    Error, Result,
};
#[cfg(feature = "loader")]
use crate::boa_backend::loader::{Loader, LoaderOpaque, Resolver};

#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
    pub contexts: usize,
}

#[derive(Default)]
struct RuntimeInner {
    contexts: RefCell<Vec<RegisteredContext>>,
    loader: SharedLoaderState,
}

#[derive(Clone)]
struct RegisteredContext {
    context: rc::Weak<RefCell<BoaContext>>,
    exception_state: rc::Weak<RefCell<Option<boa_engine::JsValue>>>,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct WeakRuntime(Weak<RuntimeInner>);

impl WeakRuntime {
    pub fn try_ref(&self) -> Option<Runtime> {
        self.0.upgrade().map(|inner| Runtime { inner })
    }

    pub(crate) fn ptr_eq(&self, other: &Runtime) -> bool {
        self.0.ptr_eq(&Arc::downgrade(&other.inner))
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Runtime {
    inner: Arc<RuntimeInner>,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(RuntimeInner::default()),
        })
    }

    pub fn weak(&self) -> WeakRuntime {
        WeakRuntime(Arc::downgrade(&self.inner))
    }

    pub fn set_info<S>(&self, _info: S) -> Result<()>
    where
        S: Into<alloc::vec::Vec<u8>>,
    {
        Ok(())
    }

    pub fn set_memory_limit(&self, _limit: usize) {}

    pub fn set_max_stack_size(&self, _limit: usize) {}

    pub fn set_gc_threshold(&self, _threshold: usize) {}

    pub fn run_gc(&self) {}

    pub fn memory_usage(&self) -> MemoryUsage {
        MemoryUsage {
            contexts: self.prune_contexts(),
        }
    }

    pub fn execute_pending_job(&self) -> core::result::Result<bool, Error> {
        let mut ran_any = false;
        let mut saw_exception = false;
        self.inner.contexts.borrow_mut().retain(|registered| {
            let Some(inner) = registered.context.upgrade() else {
                return false;
            };

            let mut boa = inner.borrow_mut();
            let result = boa.run_jobs();
            let ran_here = boa
                .downcast_job_executor::<CompatJobExecutor>()
                .map(|executor| executor.take_last_run_executed())
                .unwrap_or(false);
            ran_any |= ran_here;

            if let Err(error) = result {
                if let Some(exception_state) = registered.exception_state.upgrade() {
                    let value = error.to_opaque(&mut boa);
                    *exception_state.borrow_mut() = Some(value);
                }
                saw_exception = true;
            }

            true
        });

        if saw_exception {
            return Err(Error::Exception);
        }

        Ok(ran_any)
    }

    #[cfg(feature = "loader")]
    pub fn set_loader<R, L>(&self, resolver: R, loader: L)
    where
        R: Resolver + Send + 'static,
        L: Loader + Send + 'static,
    {
        if let Ok(mut state) = self.inner.loader.lock() {
            *state = Some(LoaderOpaque {
                resolver: Box::new(resolver),
                loader: Box::new(loader),
            });
        }
    }

    pub(crate) fn register_context(
        &self,
        context: &rc::Rc<RefCell<BoaContext>>,
        exception_state: &rc::Rc<RefCell<Option<boa_engine::JsValue>>>,
    ) {
        self.inner.contexts.borrow_mut().push(RegisteredContext {
            context: rc::Rc::downgrade(context),
            exception_state: rc::Rc::downgrade(exception_state),
        });
    }

    fn prune_contexts(&self) -> usize {
        let mut contexts = self.inner.contexts.borrow_mut();
        contexts.retain(|registered| registered.context.strong_count() > 0);
        contexts.len()
    }

    #[allow(dead_code)]
    pub(crate) fn loader_state(&self) -> SharedLoaderState {
        self.inner.loader.clone()
    }
}
