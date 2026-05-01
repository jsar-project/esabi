use core::{
    fmt,
    mem::{self, ManuallyDrop},
};

use crate::{Ctx, Error, JsLifetime, Result};

#[derive(Clone)]
pub struct Persistent<T> {
    runtime: crate::runtime::WeakRuntime,
    value: T,
}

impl<T> fmt::Debug for Persistent<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Persistent")
            .field("value", &self.value)
            .finish()
    }
}

impl<T> Persistent<T> {
    unsafe fn outlive_transmute<'from, 'to, U>(t: U) -> U::Changed<'to>
    where
        U: JsLifetime<'from>,
    {
        assert_eq!(mem::size_of::<U>(), mem::size_of::<U::Changed<'static>>());
        assert_eq!(mem::align_of::<U>(), mem::align_of::<U::Changed<'static>>());

        union Transmute<A, B> {
            a: ManuallyDrop<A>,
            b: ManuallyDrop<B>,
        }
        let data = Transmute::<U, U::Changed<'to>> {
            a: ManuallyDrop::new(t),
        };
        unsafe { ManuallyDrop::into_inner(data.b) }
    }

    pub fn save<'js>(ctx: &Ctx<'js>, val: T) -> Persistent<T::Changed<'static>>
    where
        T: JsLifetime<'js>,
    {
        let outlived = unsafe { Self::outlive_transmute::<'js, 'static, T>(val) };
        Persistent {
            runtime: ctx.runtime().weak(),
            value: outlived,
        }
    }

    pub fn restore<'js>(self, ctx: &Ctx<'js>) -> Result<T::Changed<'js>>
    where
        T: JsLifetime<'static>,
    {
        if !self.runtime.ptr_eq(ctx.runtime()) {
            return Err(Error::new_from_js_message(
                "Persistent",
                "value",
                "UnrelatedRuntime",
            ));
        }
        Ok(unsafe { Self::outlive_transmute::<'static, 'js, T>(self.value) })
    }
}
