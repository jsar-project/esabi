use alloc::{boxed::Box, format, string::String as StdString, vec, vec::Vec};
use core::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use boa_engine::{
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    property::PropertyDescriptor,
    JsString,
    JsValue,
};

use crate::{
    boa_backend::{
        class::{Class, JsClass},
        context::clone_exception_state,
        object::AsProperty,
        value::{FromJs, IntoJs, Object, Type, Value},
    },
    Ctx, Error, JsLifetime, Result,
};

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Function<'js>(pub(crate) Object<'js>);

impl<'js> Function<'js> {
    pub fn new<P, F>(ctx: Ctx<'js>, f: F) -> Result<Self>
    where
        F: IntoBoaFunction<'js, P> + 'static,
    {
        let function = build_function_object(&ctx, f, false)?;
        Ok(Self(Object::from_boa_object(ctx, function.into())))
    }

    pub fn from_object(object: Object<'js>) -> Result<Self> {
        if !object.is_function() {
            return Err(Error::new_from_js("object", "function"));
        }
        Ok(Self(object))
    }

    pub fn call<A, R>(&self, args: A) -> Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        let ctx = self.ctx().clone();
        let callable = self
            .0
            .as_value()
            .clone()
            .into_inner()
            .as_callable()
            .ok_or_else(|| Error::new_from_js("object", "function"))?;
        let (this, args) = args.into_call_args(&ctx)?;
        let value = ctx
            .with_boa(|boa| callable.call(&this, args.as_slice(), boa))
            .map_err(|err| ctx.store_exception(err))?;
        R::from_js(&ctx, Value::from_boa(ctx.clone(), value))
    }

    pub fn into_value(self) -> Value<'js> {
        self.0.into_value()
    }

    pub fn into_object(self) -> Object<'js> {
        self.0
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        self.0.ctx()
    }

    pub fn is_constructor(&self) -> bool {
        self.0.is_constructor()
    }

    pub fn set_name<S: AsRef<str>>(&self, name: S) -> Result<()> {
        let function = self
            .0
            .as_value()
            .clone()
            .into_inner()
            .as_object()
            .ok_or_else(|| Error::new_from_js("value", "object"))?;
        let name = JsString::from(name.as_ref());
        self.ctx()
            .with_boa(|boa| {
                function.define_property_or_throw(
                    js_string!("name"),
                    PropertyDescriptor::builder()
                        .value(name)
                        .writable(false)
                        .enumerable(false)
                        .configurable(true)
                        .build(),
                    boa,
                )
            })
            .map_err(|err| self.ctx().store_exception(err))?;
        Ok(())
    }

    pub fn with_name<S: AsRef<str>>(self, name: S) -> Result<Self> {
        self.set_name(name)?;
        Ok(self)
    }
}

pub fn from_js_func<'js, T, P>(ctx: Ctx<'js>, f: T) -> Result<Function<'js>>
where
    T: IntoJsFunc<'js, P> + 'static,
{
    let ptr = Box::into_raw(Box::new(f)) as usize;
    let exception_state_ptr = alloc::rc::Rc::as_ptr(&ctx.exception_state) as usize;
    let function = ctx
        .with_boa(|boa| -> boa_engine::JsResult<_> {
        let native = NativeFunction::from_copy_closure(move |this, args, boa_ctx| {
            let call_ctx = unsafe {
                change_js_lifetime(Ctx::from_borrowed_with_state(
                    boa_ctx,
                    clone_exception_state(exception_state_ptr),
                ))
            };
            let function = unsafe { &*(ptr as *const T) };
            let args = args
                .iter()
                .cloned()
                .map(|value| Value::from_boa(call_ctx.clone(), value))
                .collect::<Vec<_>>();
            let this = Value::from_boa(call_ctx.clone(), this.clone());
            let params = Params::new(call_ctx.clone(), this, args.as_slice());
            params
                .check_params(T::param_requirements())
                .map_err(crate::boa_backend::module::to_boa_error)?;
            let result = function.call(params).map_err(crate::boa_backend::module::to_boa_error)?;
            Ok(result.into_inner())
        });
        let function = FunctionObjectBuilder::new(boa.realm(), native)
            .length(T::param_requirements().min)
            .constructor(false)
            .build();
        Ok(function)
    })
        .map_err(|err| ctx.store_exception(err))?;
    Function::from_object(Object::from_boa_object(ctx, function.into()))
}

pub fn new_class_from_js_func<'js, C, T, P>(ctx: Ctx<'js>, f: T) -> Result<Constructor<'js>>
where
    for<'any> C: JsClass<'any>,
    T: IntoJsFunc<'js, P> + 'static,
{
    let ptr = Box::into_raw(Box::new(f)) as usize;
    let exception_state_ptr = alloc::rc::Rc::as_ptr(&ctx.exception_state) as usize;
    let function = ctx
        .with_boa(|boa| -> boa_engine::JsResult<_> {
            let native = NativeFunction::from_copy_closure(move |this, args, boa_ctx| {
                let call_ctx = unsafe {
                    change_js_lifetime(Ctx::from_borrowed_with_state(
                        boa_ctx,
                        clone_exception_state(exception_state_ptr),
                    ))
                };
                let function = unsafe { &*(ptr as *const T) };
                let args = args
                    .iter()
                    .cloned()
                    .map(|value| Value::from_boa(call_ctx.clone(), value))
                    .collect::<Vec<_>>();
                let this = Value::from_boa(call_ctx.clone(), this.clone());
                let params = Params::new(call_ctx.clone(), this, args.as_slice());
                params
                    .check_params(T::param_requirements())
                    .map_err(crate::boa_backend::module::to_boa_error)?;
                let result = function.call(params).map_err(crate::boa_backend::module::to_boa_error)?;
                Ok(result.into_inner())
            });
            let function = FunctionObjectBuilder::new(boa.realm(), native)
                .length(T::param_requirements().min)
                .constructor(true)
                .build();
            Ok(function)
        })
        .map_err(|err| ctx.store_exception(err))?;
    let function = Function::from_object(Object::from_boa_object(ctx.clone(), function.into()))?
        .with_name(C::NAME)?;
    if let Some(prototype) = Class::<C>::prototype(&ctx)? {
        let function_object = function.into_object().as_boa_object()?;
        let prototype_value = prototype.into_value().into_inner();
        ctx.with_boa(|boa| function_object.set(js_string!("prototype"), prototype_value, true, boa))
            .map_err(|err| ctx.store_exception(err))?;
        return Ok(Constructor(Function::from_object(Object::from_boa_object(
            ctx,
            function_object,
        ))?));
    }
    Ok(Constructor(function))
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Constructor<'js>(pub(crate) Function<'js>);

impl<'js> Constructor<'js> {
    pub fn new_class<C, P, F>(ctx: Ctx<'js>, f: F) -> Result<Self>
    where
        for<'any> C: JsClass<'any>,
        F: IntoBoaFunction<'js, P> + 'static,
    {
        let function = build_function_object(&ctx, f, true)?;
        let function = Function::from_object(Object::from_boa_object(ctx.clone(), function.clone().into()))?
            .with_name(C::NAME)?;
        if let Some(prototype) = Class::<C>::prototype(&ctx)? {
            let function_object = function.into_object().as_boa_object()?;
            let prototype_value = prototype.into_value().into_inner();
            ctx.with_boa(|boa| {
                function_object
                    .set(js_string!("prototype"), prototype_value, true, boa)
            })
            .map_err(|err| ctx.store_exception(err))?;
            return Ok(Self(Function::from_object(Object::from_boa_object(
                ctx,
                function_object,
            ))?));
        }
        Ok(Self(function))
    }

    pub fn construct<A, R>(&self, args: A) -> Result<R>
    where
        A: IntoArgs<'js>,
        R: FromJs<'js>,
    {
        let ctx = self.ctx().clone();
        let constructor = self
            .0
            .clone()
            .into_value()
            .into_inner()
            .as_constructor()
            .ok_or_else(|| Error::new_from_js("object", "constructor"))?;
        let (_this, args) = args.into_call_args(&ctx)?;
        let value = ctx
            .with_boa(|boa| constructor.construct(args.as_slice(), Some(&constructor), boa))
            .map_err(|err| ctx.store_exception(err))?;
        R::from_js(&ctx, Value::from_boa(ctx.clone(), value.into()))
    }

    pub fn prop<K, V>(&self, key: K, prop: V) -> Result<()>
    where
        K: crate::boa_backend::value::IntoPropKey,
        V: AsProperty<'js>,
    {
        self.0.clone().into_object().prop(key, prop)
    }

    pub fn into_function(self) -> Function<'js> {
        self.0
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        self.0.ctx()
    }
}

impl<'js> FromJs<'js> for Constructor<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let function = Function::from_js(_ctx, value)?;
        if function.is_constructor() {
            Ok(Self(function))
        } else {
            Err(Error::new_from_js("function", "constructor"))
        }
    }
}

impl<'js> IntoJs<'js> for Constructor<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.0.into_js(ctx)
    }
}

impl<'js> FromJs<'js> for Function<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.type_of() != Type::Function {
            return Err(Error::new_from_js(value.type_name(), "function"));
        }
        Self::from_object(
            value
                .into_object()
                .ok_or_else(|| Error::new_from_js("value", "function"))?,
        )
    }
}

impl<'js> IntoJs<'js> for Function<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value())
    }
}

pub struct This<T>(pub T);
pub struct FuncArg<T>(pub T);
pub struct Opt<T>(pub Option<T>);
pub struct Rest<T>(pub Vec<T>);
pub struct Flat<T>(pub T);
pub struct Exhaustive;
pub struct MutFn<T>(pub RefCell<T>);
pub struct OnceFn<T>(pub Cell<Option<T>>);

/// Helper type to turn Rust closures into JS-callable function values.
pub struct Func<T, P>(T, PhantomData<P>);

impl<'js, T, P> Func<T, P>
where
    T: IntoBoaFunction<'js, P>,
{
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<'js, T, P> From<T> for Func<T, P>
where
    T: IntoBoaFunction<'js, P>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

macro_rules! type_impls {
    ($($type:ident<$param:ident>;)*) => {
        $(
            impl<$param> From<$param> for $type<$param> {
                fn from(value: $param) -> Self {
                    Self(value)
                }
            }

            impl<$param> Deref for $type<$param> {
                type Target = $param;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<$param> DerefMut for $type<$param> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        )*
    };
}

type_impls! {
    This<T>;
    FuncArg<T>;
    Flat<T>;
}

impl<T> From<Option<T>> for Opt<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> Deref for Opt<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Opt<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for Rest<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> Deref for Rest<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Rest<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> MutFn<T> {
    pub fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }
}

impl<T> From<T> for MutFn<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> OnceFn<T> {
    pub fn new(value: T) -> Self {
        Self(Cell::new(Some(value)))
    }
}

impl<T> From<T> for OnceFn<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

pub struct Params<'a, 'js> {
    ctx: Ctx<'js>,
    this: Value<'js>,
    args: &'a [Value<'js>],
}

impl<'a, 'js> Params<'a, 'js> {
    fn new(ctx: Ctx<'js>, this: Value<'js>, args: &'a [Value<'js>]) -> Self {
        Self { ctx, this, args }
    }

    pub fn check_params(&self, req: ParamRequirement) -> Result<()> {
        if self.args.len() < req.min {
            return Err(Error::new_from_js_message(
                "arguments",
                "params",
                format!("expected at least {}, got {}", req.min, self.args.len()),
            ));
        }
        if req.exhaustive && self.args.len() > req.max {
            return Err(Error::new_from_js_message(
                "arguments",
                "params",
                format!("expected at most {}, got {}", req.max, self.args.len()),
            ));
        }
        Ok(())
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    pub fn this(&self) -> Value<'js> {
        self.this.clone()
    }

    pub fn function(&self) -> Value<'js> {
        Value::new_undefined(self.ctx.clone())
    }

    pub fn arg(&self, index: usize) -> Option<Value<'js>> {
        self.args.get(index).cloned()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn access(self) -> ParamsAccessor<'a, 'js> {
        ParamsAccessor {
            params: self,
            offset: 0,
        }
    }
}

pub struct ParamsAccessor<'a, 'js> {
    params: Params<'a, 'js>,
    offset: usize,
}

impl<'a, 'js> ParamsAccessor<'a, 'js> {
    pub fn ctx(&self) -> &Ctx<'js> {
        self.params.ctx()
    }

    pub fn this(&self) -> Value<'js> {
        self.params.this()
    }

    pub fn function(&self) -> Value<'js> {
        self.params.function()
    }

    pub fn arg(&mut self) -> Value<'js> {
        let value = self
            .params
            .args
            .get(self.offset)
            .cloned()
            .unwrap_or_else(|| Value::new_undefined(self.params.ctx.clone()));
        self.offset += 1;
        value
    }

    pub fn len(&self) -> usize {
        self.params.args.len().saturating_sub(self.offset)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Copy)]
pub struct ParamRequirement {
    min: usize,
    max: usize,
    exhaustive: bool,
}

impl ParamRequirement {
    pub const fn none() -> Self {
        Self {
            min: 0,
            max: 0,
            exhaustive: false,
        }
    }

    pub const fn single() -> Self {
        Self {
            min: 1,
            max: 1,
            exhaustive: false,
        }
    }

    pub const fn optional() -> Self {
        Self {
            min: 0,
            max: 1,
            exhaustive: false,
        }
    }

    pub const fn any() -> Self {
        Self {
            min: 0,
            max: usize::MAX,
            exhaustive: false,
        }
    }

    pub const fn exhaustive() -> Self {
        Self {
            min: 0,
            max: 0,
            exhaustive: true,
        }
    }

    pub const fn combine(self, other: Self) -> Self {
        Self {
            min: self.min.saturating_add(other.min),
            max: self.max.saturating_add(other.max),
            exhaustive: self.exhaustive || other.exhaustive,
        }
    }
}

pub trait FromParam<'js>: Sized {
    fn param_requirement() -> ParamRequirement;
    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self>;
}

impl<'js, T> FromParam<'js> for T
where
    T: FromJs<'js>,
{
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let ctx = params.ctx().clone();
        T::from_js(&ctx, params.arg())
    }
}

impl<'js> FromParam<'js> for Ctx<'js> {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::none()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(params.ctx().clone())
    }
}

impl<'js, T> FromParam<'js> for This<T>
where
    T: FromJs<'js>,
{
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::none()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let ctx = params.ctx().clone();
        Ok(Self(T::from_js(&ctx, params.this())?))
    }
}

impl<'js, T> FromParam<'js> for FuncArg<T>
where
    T: FromJs<'js>,
{
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::none()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let ctx = params.ctx().clone();
        Ok(Self(T::from_js(&ctx, params.function())?))
    }
}

impl<'js, T> FromParam<'js> for Opt<T>
where
    T: FromJs<'js>,
{
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::optional()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        if params.is_empty() {
            Ok(Self(None))
        } else {
            let ctx = params.ctx().clone();
            Ok(Self(Some(T::from_js(&ctx, params.arg())?)))
        }
    }
}

impl<'js, T> FromParam<'js> for Rest<T>
where
    T: FromJs<'js>,
{
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::any()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let ctx = params.ctx().clone();
        let mut items = Vec::with_capacity(params.len());
        while !params.is_empty() {
            items.push(T::from_js(&ctx, params.arg())?);
        }
        Ok(Self(items))
    }
}

impl<'js, T> FromParam<'js> for Flat<T>
where
    T: FromParam<'js>,
{
    fn param_requirement() -> ParamRequirement {
        T::param_requirement()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Self(T::from_param(params)?))
    }
}

impl<'js> FromParam<'js> for Exhaustive {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::exhaustive()
    }

    fn from_param<'a>(_params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Self)
    }
}

pub trait IntoJsFunc<'js, P> {
    fn param_requirements() -> ParamRequirement;
    fn call<'a>(&self, params: Params<'a, 'js>) -> Result<Value<'js>>;
}

impl<'js, T, P> IntoJs<'js> for Func<T, P>
where
    T: IntoBoaFunction<'js, P> + 'static,
{
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Function::new(ctx.clone(), self.0).map(Function::into_value)
    }
}

unsafe fn change_js_lifetime<'from, 'to, T>(value: T) -> T::Changed<'to>
where
    T: JsLifetime<'from>,
{
    let value = core::mem::ManuallyDrop::new(value);
    core::ptr::read((&*value as *const T).cast::<T::Changed<'to>>())
}

pub trait IntoFunctionValue<'js> {
    fn into_function_value(self, ctx: &Ctx<'js>) -> Result<Value<'js>>;
}

pub fn into_function_value<'js, T>(ctx: &Ctx<'js>, value: T) -> Result<Value<'js>>
where
    T: IntoFunctionValue<'js>,
{
    value.into_function_value(ctx)
}

impl<'js, T> IntoFunctionValue<'js> for T
where
    T: IntoJs<'js>,
{
    fn into_function_value(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.into_js(ctx)
    }
}

impl<'js, T> IntoFunctionValue<'js> for Result<T>
where
    T: IntoJs<'js>,
{
    fn into_function_value(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self?.into_js(ctx)
    }
}

pub trait IntoBoaFunction<'js, P>: Sized {
    const LENGTH: usize;

    fn invoke(&self, ctx: &Ctx<'js>, this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>>;
}

pub trait NotCtx {}

impl NotCtx for bool {}
impl NotCtx for i8 {}
impl NotCtx for i16 {}
impl NotCtx for i32 {}
impl NotCtx for i64 {}
impl NotCtx for isize {}
impl NotCtx for u8 {}
impl NotCtx for u16 {}
impl NotCtx for u32 {}
impl NotCtx for u64 {}
impl NotCtx for usize {}
impl NotCtx for f32 {}
impl NotCtx for f64 {}
impl NotCtx for StdString {}
impl<'js> NotCtx for Value<'js> {}
impl<'js> NotCtx for Object<'js> {}
impl<'js> NotCtx for Function<'js> {}
impl<T> NotCtx for Option<T> where T: NotCtx {}

fn this_or_undefined<'js, T>(ctx: &Ctx<'js>, this: &Value<'js>) -> Result<This<T>>
where
    T: FromJs<'js>,
{
    Ok(This(T::from_js(ctx, this.clone())?))
}

fn arg_or_undefined<'js, T>(ctx: &Ctx<'js>, args: &[Value<'js>], index: usize) -> Result<T>
where
    T: FromJs<'js>,
{
    let value = args
        .get(index)
        .cloned()
        .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
    T::from_js(ctx, value)
}

impl<'js, F, Ret> IntoBoaFunction<'js, ()> for F
where
    F: Fn() -> Ret,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 0;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, _args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)().into_function_value(ctx)
    }
}

impl<'js, F, Ret> IntoBoaFunction<'js, (Ctx<'js>,)> for F
where
    F: Fn(Ctx<'js>) -> Ret,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 0;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, _args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(ctx.clone()).into_function_value(ctx)
    }
}

impl<'js, F, A, Ret> IntoBoaFunction<'js, (This<A>,)> for F
where
    F: Fn(This<A>) -> Ret,
    A: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 0;

    fn invoke(&self, ctx: &Ctx<'js>, this: Value<'js>, _args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(this_or_undefined(ctx, &this)?).into_function_value(ctx)
    }
}

impl<'js, F, A, Ret> IntoBoaFunction<'js, (Ctx<'js>, This<A>)> for F
where
    F: Fn(Ctx<'js>, This<A>) -> Ret,
    A: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 0;

    fn invoke(&self, ctx: &Ctx<'js>, this: Value<'js>, _args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(ctx.clone(), this_or_undefined(ctx, &this)?).into_function_value(ctx)
    }
}

impl<'js, F, A, Ret> IntoBoaFunction<'js, (A,)> for F
where
    F: Fn(A) -> Ret,
    A: FromJs<'js> + NotCtx,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 1;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(arg_or_undefined(ctx, args, 0)?).into_function_value(ctx)
    }
}

impl<'js, F, A, Ret> IntoBoaFunction<'js, (Ctx<'js>, A)> for F
where
    F: Fn(Ctx<'js>, A) -> Ret,
    A: FromJs<'js> + NotCtx,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 1;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(ctx.clone(), arg_or_undefined(ctx, args, 0)?).into_function_value(ctx)
    }
}

impl<'js, F, A, B, Ret> IntoBoaFunction<'js, (This<A>, B)> for F
where
    F: Fn(This<A>, B) -> Ret,
    A: FromJs<'js>,
    B: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 1;

    fn invoke(&self, ctx: &Ctx<'js>, this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(this_or_undefined(ctx, &this)?, arg_or_undefined(ctx, args, 0)?)
            .into_function_value(ctx)
    }
}

impl<'js, F, A, B, Ret> IntoBoaFunction<'js, (Ctx<'js>, This<A>, B)> for F
where
    F: Fn(Ctx<'js>, This<A>, B) -> Ret,
    A: FromJs<'js>,
    B: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 1;

    fn invoke(&self, ctx: &Ctx<'js>, this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(
            ctx.clone(),
            this_or_undefined(ctx, &this)?,
            arg_or_undefined(ctx, args, 0)?,
        )
        .into_function_value(ctx)
    }
}

impl<'js, F, A, B, Ret> IntoBoaFunction<'js, (A, B)> for F
where
    F: Fn(A, B) -> Ret,
    A: FromJs<'js> + NotCtx,
    B: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 2;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(arg_or_undefined(ctx, args, 0)?, arg_or_undefined(ctx, args, 1)?)
            .into_function_value(ctx)
    }
}

impl<'js, F, A, B, Ret> IntoBoaFunction<'js, (Ctx<'js>, A, B)> for F
where
    F: Fn(Ctx<'js>, A, B) -> Ret,
    A: FromJs<'js> + NotCtx,
    B: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 2;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(
            ctx.clone(),
            arg_or_undefined(ctx, args, 0)?,
            arg_or_undefined(ctx, args, 1)?,
        )
        .into_function_value(ctx)
    }
}

impl<'js, F, A, B, C, Ret> IntoBoaFunction<'js, (A, B, C)> for F
where
    F: Fn(A, B, C) -> Ret,
    A: FromJs<'js> + NotCtx,
    B: FromJs<'js>,
    C: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 3;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(
            arg_or_undefined(ctx, args, 0)?,
            arg_or_undefined(ctx, args, 1)?,
            arg_or_undefined(ctx, args, 2)?,
        )
        .into_function_value(ctx)
    }
}

impl<'js, F, A, B, C, Ret> IntoBoaFunction<'js, (Ctx<'js>, A, B, C)> for F
where
    F: Fn(Ctx<'js>, A, B, C) -> Ret,
    A: FromJs<'js> + NotCtx,
    B: FromJs<'js>,
    C: FromJs<'js>,
    Ret: IntoFunctionValue<'js>,
{
    const LENGTH: usize = 3;

    fn invoke(&self, ctx: &Ctx<'js>, _this: Value<'js>, args: &[Value<'js>]) -> Result<Value<'js>> {
        (self)(
            ctx.clone(),
            arg_or_undefined(ctx, args, 0)?,
            arg_or_undefined(ctx, args, 1)?,
            arg_or_undefined(ctx, args, 2)?,
        )
        .into_function_value(ctx)
    }
}

pub trait IntoArgs<'js> {
    fn into_call_args(self, ctx: &Ctx<'js>) -> Result<(JsValue, Vec<JsValue>)>;
}

impl<'js> IntoArgs<'js> for () {
    fn into_call_args(self, _ctx: &Ctx<'js>) -> Result<(JsValue, Vec<JsValue>)> {
        Ok((JsValue::undefined(), Vec::new()))
    }
}

impl<'js, T> IntoArgs<'js> for (T,)
where
    T: IntoJs<'js>,
{
    fn into_call_args(self, ctx: &Ctx<'js>) -> Result<(JsValue, Vec<JsValue>)> {
        Ok((JsValue::undefined(), vec![self.0.into_js(ctx)?.into_inner()]))
    }
}

impl<'js, A, B> IntoArgs<'js> for (A, B)
where
    A: IntoJs<'js>,
    B: IntoJs<'js>,
{
    fn into_call_args(self, ctx: &Ctx<'js>) -> Result<(JsValue, Vec<JsValue>)> {
        Ok((
            JsValue::undefined(),
            vec![self.0.into_js(ctx)?.into_inner(), self.1.into_js(ctx)?.into_inner()],
        ))
    }
}

impl<'js, A, B, C> IntoArgs<'js> for (A, B, C)
where
    A: IntoJs<'js>,
    B: IntoJs<'js>,
    C: IntoJs<'js>,
{
    fn into_call_args(self, ctx: &Ctx<'js>) -> Result<(JsValue, Vec<JsValue>)> {
        Ok((
            JsValue::undefined(),
            vec![
                self.0.into_js(ctx)?.into_inner(),
                self.1.into_js(ctx)?.into_inner(),
                self.2.into_js(ctx)?.into_inner(),
            ],
        ))
    }
}

impl<'js, T> IntoArgs<'js> for (This<Object<'js>>, T)
where
    T: IntoJs<'js>,
{
    fn into_call_args(self, ctx: &Ctx<'js>) -> Result<(JsValue, Vec<JsValue>)> {
        Ok((
            self.0 .0.into_value().into_inner(),
            vec![self.1.into_js(ctx)?.into_inner()],
        ))
    }
}

fn build_function_object<'js, P, F>(
    ctx: &Ctx<'js>,
    f: F,
    constructor: bool,
) -> Result<JsObject>
where
    F: IntoBoaFunction<'js, P> + 'static,
{
    let ptr = Box::into_raw(Box::new(f)) as usize;
    let exception_state_ptr = alloc::rc::Rc::as_ptr(&ctx.exception_state) as usize;
    let function = ctx.with_boa(|boa| {
        let native = NativeFunction::from_copy_closure(move |this, args, boa_ctx| {
            let call_ctx = unsafe {
                change_js_lifetime(Ctx::from_borrowed_with_state(
                    boa_ctx,
                    clone_exception_state(exception_state_ptr),
                ))
            };
            let this = Value::from_boa(call_ctx.clone(), this.clone());
            let args = args
                .iter()
                .cloned()
                .map(|arg| Value::from_boa(call_ctx.clone(), arg))
                .collect::<Vec<_>>();
            let callback = unsafe { &*(ptr as *const F) };
            match callback.invoke(&call_ctx, this, &args) {
                Ok(value) => Ok(value.into_inner()),
                Err(Error::Exception) => {
                    Err(boa_engine::JsError::from_opaque(call_ctx.catch().into_inner()))
                }
                Err(error) => Err(crate::boa_backend::module::to_boa_error(error)),
            }
        });
        FunctionObjectBuilder::new(boa.realm(), native)
            .name(js_string!("anonymous"))
            .length(F::LENGTH)
            .constructor(constructor)
            .build()
            .into()
    });
    Ok(function)
}
