use core::{
    mem::ManuallyDrop,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use boa_engine::{
    class::Class as BoaClass,
    context::intrinsics::StandardConstructor,
    js_string,
    object::{JsData, JsObject, Ref as BoaRef, RefMut as BoaRefMut},
    property::Attribute,
    Context as BoaContext, Finalize, JsNativeError, JsResult, JsValue, Trace as BoaTrace,
};

use crate::{
    boa_backend::{
        context::clone_exception_state,
        function::Constructor,
        value::{FromJs, IntoJs, Object, Value},
    },
    Ctx, Error, JsLifetime, Result,
};

pub trait Trace<'js> {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[derive(Clone, Copy, Debug)]
pub struct Tracer<'a, 'js>(PhantomData<&'a ()>, PhantomData<&'js ()>);

macro_rules! trace_impls_primitive {
    ($($ty:ty),* $(,)?) => {
        $(impl<'js> Trace<'js> for $ty {})*
    };
}

trace_impls_primitive!(
    (),
    bool,
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
    alloc::string::String,
);

impl<'js, T> Trace<'js> for Option<T>
where
    T: Trace<'js>,
{
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Some(value) = self {
            value.trace(tracer);
        }
    }
}

impl<'js, T> Trace<'js> for alloc::vec::Vec<T>
where
    T: Trace<'js>,
{
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        for value in self {
            value.trace(tracer);
        }
    }
}

impl<'js> Trace<'js> for Value<'js> {}
impl<'js> Trace<'js> for Object<'js> {}

pub trait JsClass<'js>: Sized + 'static {
    const NAME: &'static str;
    type Mutable: Mutability;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        Object::new(ctx.clone()).map(Some)
    }

    fn init_prototype(ctx: &Ctx<'js>, target: &Object<'js>) -> Result<()> {
        if let Some(source) = Self::prototype(ctx)? {
            install_properties_from_object(ctx, &source, target)?;
        }
        Ok(())
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>>;
}

pub(crate) struct NativeClass<C> {
    pub(crate) value: C,
}

impl<C> Finalize for NativeClass<C> {}

// SAFETY: This minimal Boa bridge currently treats stored Rust data as non-tracing. This keeps the
// class path on native Boa objects without depending on JS-side shims. Types containing JS handles
// still need a dedicated trace bridge before they can be considered fully supported.
unsafe impl<C> BoaTrace for NativeClass<C> {
    boa_engine::gc::empty_trace!();
}

impl<C> JsData for NativeClass<C> {}

impl<C> BoaClass for NativeClass<C>
where
    for<'js> C: JsClass<'js>,
{
    const NAME: &'static str = C::NAME;
    const ATTRIBUTES: Attribute = Attribute::all();

    fn init(_class: &mut boa_engine::class::ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut BoaContext,
    ) -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("native Boa class constructors must be created through esabi::function::Constructor::new_class")
            .into())
    }
}

#[derive(Debug)]
pub struct Class<'js, C: 'static> {
    object: Object<'js>,
    boa_object: JsObject,
    _marker: PhantomData<C>,
}

impl<'js, C: 'static> Clone for Class<'js, C> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            boa_object: self.boa_object.clone(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'js, C> JsLifetime<'js> for Class<'js, C>
where
    C: 'static,
{
    type Changed<'to> = Class<'to, C>;
}

impl<'js, C: 'static> Deref for Class<'js, C> {
    type Target = Object<'js>;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

pub struct Borrow<'a, C: 'static>(BoaRef<'a, NativeClass<C>>);

impl<C> Deref for Borrow<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

pub struct BorrowMut<'a, C: 'static>(BoaRefMut<'a, NativeClass<C>>);

impl<C> Deref for BorrowMut<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<C> DerefMut for BorrowMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.value
    }
}

pub unsafe trait Mutability {
    const WRITABLE: bool;
}

pub enum Readable {}
pub enum Writable {}

unsafe impl Mutability for Readable {
    const WRITABLE: bool = false;
}

unsafe impl Mutability for Writable {
    const WRITABLE: bool = true;
}

pub struct OwnedBorrow<'js, C: 'static> {
    class: ManuallyDrop<Class<'js, C>>,
    borrow: BoaRef<'static, NativeClass<C>>,
}

impl<'js, C> OwnedBorrow<'js, C>
where
    C: JsClass<'js>,
{
    pub fn from_class(class: Class<'js, C>) -> Self {
        Self::try_from_class(class).expect("class should be borrowable")
    }

    pub fn try_from_class(class: Class<'js, C>) -> Result<Self> {
        let borrow = class
            .boa_object
            .downcast_ref::<NativeClass<C>>()
            .ok_or_else(|| Error::new_from_js("object", "class"))?;
        let borrow = unsafe {
            core::mem::transmute::<BoaRef<'_, NativeClass<C>>, BoaRef<'static, NativeClass<C>>>(
                borrow,
            )
        };
        Ok(Self {
            class: ManuallyDrop::new(class),
            borrow,
        })
    }

    pub fn into_inner(mut self) -> Class<'js, C> {
        let class = unsafe { ManuallyDrop::take(&mut self.class) };
        core::mem::forget(self);
        class
    }
}

impl<'js, C: 'static> Drop for OwnedBorrow<'js, C> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.class);
        }
    }
}

impl<C: 'static> Deref for OwnedBorrow<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.borrow.value
    }
}

pub struct OwnedBorrowMut<'js, C: 'static> {
    class: ManuallyDrop<Class<'js, C>>,
    borrow: BoaRefMut<'static, NativeClass<C>>,
}

impl<'js, C> OwnedBorrowMut<'js, C>
where
    C: JsClass<'js>,
{
    pub fn from_class(class: Class<'js, C>) -> Self {
        Self::try_from_class(class).expect("class should be mutably borrowable")
    }

    pub fn try_from_class(class: Class<'js, C>) -> Result<Self> {
        if !<C::Mutable as Mutability>::WRITABLE {
            return Err(Error::unsupported("class is not writable"));
        }
        let borrow = class
            .boa_object
            .downcast_mut::<NativeClass<C>>()
            .ok_or_else(|| Error::new_from_js("object", "class"))?;
        let borrow = unsafe {
            core::mem::transmute::<
                BoaRefMut<'_, NativeClass<C>>,
                BoaRefMut<'static, NativeClass<C>>,
            >(borrow)
        };
        Ok(Self {
            class: ManuallyDrop::new(class),
            borrow,
        })
    }

    pub fn into_inner(mut self) -> Class<'js, C> {
        let class = unsafe { ManuallyDrop::take(&mut self.class) };
        core::mem::forget(self);
        class
    }
}

impl<'js, C: 'static> Drop for OwnedBorrowMut<'js, C> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.class);
        }
    }
}

impl<C: 'static> Deref for OwnedBorrowMut<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.borrow.value
    }
}

impl<C: 'static> DerefMut for OwnedBorrowMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.borrow.value
    }
}

impl<'js, C> Class<'js, C>
where
    for<'any> C: JsClass<'any>,
{
    pub fn instance(ctx: Ctx<'js>, value: C) -> Result<Self> {
        ensure_native_class::<C>(&ctx)?;
        let object = ctx
            .with_boa(|boa| NativeClass::<C>::from_data(NativeClass { value }, boa))
            .map_err(|err| ctx.store_exception(err))?;
        Ok(Self {
            object: Object::from_boa_object(ctx, object.clone()),
            boa_object: object,
            _marker: PhantomData,
        })
    }

    pub fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let native = ensure_native_class::<C>(ctx)?;
        Ok(Some(Object::from_boa_object(
            ctx.clone(),
            native.prototype(),
        )))
    }

    pub fn create_constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        C::constructor(ctx)
    }

    pub fn define(object: &Object<'js>) -> Result<()> {
        if let Some(constructor) = Self::create_constructor(object.ctx())? {
            object.set(C::NAME, constructor)?;
        }
        Ok(())
    }
}

impl<'js, C> Class<'js, C>
where
    C: 'static,
{
    pub fn from_object(object: Object<'js>) -> Result<Self> {
        let boa_object = object.as_boa_object()?;
        if boa_object.downcast_ref::<NativeClass<C>>().is_some() {
            Ok(Self {
                object,
                boa_object,
                _marker: PhantomData,
            })
        } else {
            Err(Error::new_from_js("object", "class"))
        }
    }

    pub fn try_borrow(&self) -> Result<Borrow<'_, C>> {
        let borrowed = self
            .boa_object
            .downcast_ref::<NativeClass<C>>()
            .ok_or_else(|| Error::new_from_js("object", "class"))?;
        Ok(Borrow(borrowed))
    }

    pub fn borrow(&self) -> Borrow<'_, C> {
        self.try_borrow().expect("class should be borrowable")
    }

    pub fn try_borrow_mut(&self) -> Result<BorrowMut<'_, C>>
    where
        C: JsClass<'js>,
    {
        if !<C::Mutable as Mutability>::WRITABLE {
            return Err(Error::unsupported("class is not writable"));
        }
        let borrowed = self
            .boa_object
            .downcast_mut::<NativeClass<C>>()
            .ok_or_else(|| Error::new_from_js("object", "class"))?;
        Ok(BorrowMut(borrowed))
    }

    pub fn borrow_mut(&self) -> BorrowMut<'_, C>
    where
        C: JsClass<'js>,
    {
        self.try_borrow_mut()
            .expect("class should be mutably borrowable")
    }

    pub fn as_object(&self) -> &Object<'js> {
        &self.object
    }

    pub fn into_object(self) -> Object<'js> {
        self.object
    }

    pub fn into_inner(self) -> Object<'js> {
        self.into_object()
    }
}

impl<'js, C> FromJs<'js> for Class<'js, C>
where
    C: 'static,
{
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let ctx = value.ctx().clone();
        Self::from_object(Object::from_js(&ctx, value)?)
    }
}

impl<'js, C> IntoJs<'js> for Class<'js, C>
where
    C: 'static,
{
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_object().into_value())
    }
}

impl<'js, C> FromJs<'js> for OwnedBorrow<'js, C>
where
    C: JsClass<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Self::try_from_class(Class::from_js(ctx, value)?)
    }
}

impl<'js, C> IntoJs<'js> for OwnedBorrow<'js, C>
where
    C: JsClass<'js>,
{
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.into_inner().into_js(ctx)
    }
}

impl<'js, C> FromJs<'js> for OwnedBorrowMut<'js, C>
where
    C: JsClass<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Self::try_from_class(Class::from_js(ctx, value)?)
    }
}

impl<'js, C> IntoJs<'js> for OwnedBorrowMut<'js, C>
where
    C: JsClass<'js>,
{
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.into_inner().into_js(ctx)
    }
}

pub mod impl_ {
    use super::JsClass;
    use crate::{function::Constructor, Ctx, Object, Result};
    use core::marker::PhantomData;

    pub trait MethodImplementor<T>: Sized {
        fn implement<'js>(&self, _proto: &Object<'js>) -> Result<()> {
            Ok(())
        }
    }

    pub trait ConstructorCreator<'js, T>: Sized {
        fn create_constructor(&self, _ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
            Ok(None)
        }
    }

    #[derive(Default)]
    pub struct MethodImpl<T>(PhantomData<T>);

    impl<T> MethodImpl<T> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    #[derive(Default)]
    pub struct ConstructorCreate<T>(PhantomData<T>);

    impl<T> ConstructorCreate<T> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<T> MethodImplementor<T> for &MethodImpl<T> {}
    impl<'js, T> ConstructorCreator<'js, T> for &ConstructorCreate<T> {}

    pub struct CloneWrapper<'a, T>(pub &'a T);
    pub trait CloneTrait<T> {
        fn wrap_clone(&self) -> T;
    }

    impl<'a, T: Clone> CloneTrait<T> for CloneWrapper<'a, T> {
        fn wrap_clone(&self) -> T {
            self.0.clone()
        }
    }

    #[derive(Default)]
    pub struct JsClassFieldCheck<T>(PhantomData<T>);

    impl<T> JsClassFieldCheck<T> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    #[diagnostic::on_unimplemented(
        message = "using a `JsClass` type directly as a class field is not supported",
        label = "`{Self}` implements `JsClass` - wrap the field in `Class<'js, T>` instead",
        note = "nested mutations are lost because the generated getter clones the value"
    )]
    pub trait NotAJsClassField {}

    impl<'js, T: JsClass<'js>> JsClassFieldCheck<T> {
        pub fn check(self)
        where
            T: NotAJsClassField,
        {
        }
    }

    pub trait JsClassFieldCheckFallback {
        fn check(self);
    }

    impl<T> JsClassFieldCheckFallback for &JsClassFieldCheck<T> {
        fn check(self) {}
    }
}

fn ensure_native_class<'js, C>(ctx: &Ctx<'js>) -> Result<StandardConstructor>
where
    for<'any> C: JsClass<'any>,
{
    let mut prototype_to_install: Option<JsObject> = None;
    let native = ctx
        .with_boa(|boa| {
        if !boa.has_global_class::<NativeClass<C>>() {
            boa.register_global_class::<NativeClass<C>>()?;
            let native = boa
                .get_global_class::<NativeClass<C>>()
                .expect("registered Boa class should exist");
            prototype_to_install = Some(native.prototype());
            boa.global_object()
                .delete_property_or_throw(js_string!(C::NAME), boa)?;
        }

        Ok(boa
            .get_global_class::<NativeClass<C>>()
            .expect("registered Boa class should exist"))
    })
    .map_err(|err: boa_engine::JsError| ctx.store_exception(err))?;
    if let Some(prototype) = prototype_to_install {
        install_prototype::<C>(ctx, &prototype)?;
    }
    Ok(native)
}

fn install_prototype<'js, C>(ctx: &Ctx<'js>, target: &JsObject) -> Result<()>
where
    for<'any> C: JsClass<'any>,
{
    let exception_state_ptr = alloc::rc::Rc::as_ptr(&ctx.exception_state) as usize;
    ctx.with_boa(|boa| -> boa_engine::JsResult<()> {
        let borrowed = Ctx::from_borrowed_with_state(
            boa,
            unsafe { clone_exception_state(exception_state_ptr) },
        );
        let target = Object::from_boa_object(borrowed.clone(), target.clone());
        C::init_prototype(&borrowed, &target).map_err(crate::boa_backend::module::to_boa_error)
    })
    .map_err(|err: boa_engine::JsError| ctx.store_exception(err))?;
    Ok(())
}

fn install_properties_from_object<'js>(
    ctx: &Ctx<'js>,
    source: &Object<'js>,
    target: &Object<'js>,
) -> Result<()> {
    let source = source.as_boa_object()?;
    let target = target.as_boa_object()?;
    ctx.with_boa(|boa| {
        for key in source.own_property_keys(boa)? {
            let value = source.get(key.clone(), boa)?;
            target.set(key, value, true, boa)?;
        }
        Ok(())
    })
    .map_err(|err: boa_engine::JsError| ctx.store_exception(err))?;
    Ok(())
}
