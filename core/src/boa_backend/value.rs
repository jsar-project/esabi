use alloc::string::{String as StdString, ToString as _};
use core::{fmt, marker::PhantomData};

use boa_engine::{object::JsObject, property::PropertyKey, JsString, JsValue};

use crate::boa_backend::atom::PredefinedAtom;
use crate::{Ctx, Error, Result};

pub trait FromJs<'js>: Sized {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self>;
}

pub trait IntoJs<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Type {
    Undefined,
    Null,
    Bool,
    Int,
    Float,
    String,
    Array,
    Function,
    Object,
    Unknown,
}

impl Type {
    pub const fn is_void(self) -> bool {
        matches!(self, Self::Undefined | Self::Null)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Type::Undefined => "undefined",
            Type::Null => "null",
            Type::Bool => "bool",
            Type::Int => "int",
            Type::Float => "float",
            Type::String => "string",
            Type::Array => "array",
            Type::Function => "function",
            Type::Object => "object",
            Type::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Clone)]
pub struct Value<'js> {
    ctx: Ctx<'js>,
    value: JsValue,
}

impl<'js> Value<'js> {
    pub(crate) fn from_boa(ctx: Ctx<'js>, value: JsValue) -> Self {
        Self { ctx, value }
    }

    pub(crate) fn into_inner(self) -> JsValue {
        self.value
    }

    pub fn new_undefined(ctx: Ctx<'js>) -> Self {
        Self::from_boa(ctx, JsValue::undefined())
    }

    pub fn new_null(ctx: Ctx<'js>) -> Self {
        Self::from_boa(ctx, JsValue::null())
    }

    pub fn new_bool(ctx: Ctx<'js>, value: bool) -> Self {
        Self::from_boa(ctx, JsValue::from(value))
    }

    pub fn new_int(ctx: Ctx<'js>, value: i32) -> Self {
        Self::from_boa(ctx, JsValue::from(value))
    }

    pub fn new_float(ctx: Ctx<'js>, value: f64) -> Self {
        Self::from_boa(ctx, JsValue::from(value))
    }

    pub fn new_number(ctx: Ctx<'js>, value: f64) -> Self {
        Self::new_float(ctx, value)
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    pub fn type_of(&self) -> Type {
        if self.is_undefined() {
            Type::Undefined
        } else if self.is_null() {
            Type::Null
        } else if self.is_bool() {
            Type::Bool
        } else if self.is_string() {
            Type::String
        } else if self.is_int() {
            Type::Int
        } else if self.is_float() {
            Type::Float
        } else if self.is_array() {
            Type::Array
        } else if self.is_function() {
            Type::Function
        } else if self.is_object() {
            Type::Object
        } else {
            Type::Unknown
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_of().as_str()
    }

    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }

    pub fn is_undefined(&self) -> bool {
        self.value.is_undefined()
    }

    pub fn is_bool(&self) -> bool {
        self.value.is_boolean()
    }

    pub fn is_int(&self) -> bool {
        self.as_int().is_some()
    }

    pub fn is_float(&self) -> bool {
        self.value.is_number() && !self.is_int()
    }

    pub fn is_string(&self) -> bool {
        self.value.is_string()
    }

    pub fn is_object(&self) -> bool {
        self.value.is_object()
    }

    pub fn is_array(&self) -> bool {
        self.value
            .as_object()
            .map(|object| object.is_array())
            .unwrap_or(false)
    }

    pub fn is_function(&self) -> bool {
        self.value.is_callable()
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_boolean()
    }

    pub fn as_int(&self) -> Option<i32> {
        let number = self.value.as_number()?;
        if number.fract() != 0.0 {
            return None;
        }
        if number < i32::MIN as f64 || number > i32::MAX as f64 {
            return None;
        }
        Some(number as i32)
    }

    pub fn as_float(&self) -> Option<f64> {
        self.value.as_number()
    }

    pub fn as_number(&self) -> Option<f64> {
        self.value.as_number()
    }

    pub fn get<T: FromJs<'js>>(&self) -> Result<T> {
        T::from_js(self.ctx(), self.clone())
    }

    pub fn into_object(self) -> Option<Object<'js>> {
        self.value
            .as_object()
            .map(|object| Object::from_boa_object(self.ctx, object))
    }

    pub fn as_object(&self) -> Option<Object<'js>> {
        self.value
            .as_object()
            .map(|object| Object::from_boa_object(self.ctx.clone(), object))
    }
}

impl<'js> fmt::Debug for Value<'js> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.type_of() {
            Type::Undefined => f.write_str("undefined"),
            Type::Null => f.write_str("null"),
            Type::Bool => write!(f, "bool({})", self.as_bool().unwrap_or(false)),
            Type::Int => write!(f, "int({})", self.as_int().unwrap_or_default()),
            Type::Float => write!(f, "float({})", self.as_float().unwrap_or_default()),
            Type::String => {
                let result = self
                    .ctx
                    .with_boa(|boa| self.value.to_string(boa).map(|s| s.to_std_string_escaped()));
                match result {
                    Ok(value) => write!(f, "string({value:?})"),
                    Err(_) => f.write_str("string(<error>)"),
                }
            }
            Type::Array => f.write_str("array(..)"),
            Type::Function => f.write_str("function(..)"),
            Type::Object => f.write_str("object(..)"),
            Type::Unknown => f.write_str("unknown(..)"),
        }
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Object<'js>(pub(crate) Value<'js>);

impl<'js> Object<'js> {
    pub(crate) fn from_boa_object(ctx: Ctx<'js>, object: JsObject) -> Self {
        Self(Value::from_boa(ctx, object.into()))
    }

    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        let object = ctx.with_boa(|boa| JsObject::with_object_proto(boa.intrinsics()));
        Ok(Self::from_boa_object(ctx, object))
    }

    pub fn get<K, V>(&self, key: K) -> Result<V>
    where
        K: IntoPropKey,
        V: FromJs<'js>,
    {
        let key = key.into_prop_key();
        let object = self.as_boa_object()?;
        let value = self
            .0
            .ctx
            .with_boa(|boa| object.get(key, boa))
            .map_err(|err| self.0.ctx.store_exception(err))?;
        V::from_js(self.ctx(), Value::from_boa(self.ctx().clone(), value))
    }

    pub fn contains_key<K>(&self, key: K) -> Result<bool>
    where
        K: IntoPropKey,
    {
        let key = key.into_prop_key();
        let object = self.as_boa_object()?;
        self.0
            .ctx
            .with_boa(|boa| object.has_property(key, boa))
            .map_err(|err| self.0.ctx.store_exception(err))
    }

    pub fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: IntoPropKey,
        V: IntoJs<'js>,
    {
        let key = key.into_prop_key();
        let object = self.as_boa_object()?;
        let value = value.into_js(self.ctx())?;
        self.0
            .ctx
            .with_boa(|boa| object.set(key, value.value, true, boa))
            .map_err(|err| self.0.ctx.store_exception(err))?;
        Ok(())
    }

    pub fn remove<K>(&self, _key: K) -> Result<()>
    where
        K: IntoPropKey,
    {
        Err(Error::unsupported(
            "Object::remove is not implemented on the Boa backend yet",
        ))
    }

    pub fn is_array(&self) -> bool {
        self.0.is_array()
    }

    pub fn is_function(&self) -> bool {
        self.0.is_function()
    }

    pub fn is_constructor(&self) -> bool {
        self.as_boa_object()
            .map(|object| object.is_constructor())
            .unwrap_or(false)
    }

    pub fn set_prototype(&self, prototype: Option<&Object<'js>>) -> Result<()> {
        let object = self.as_boa_object()?;
        let prototype = match prototype {
            Some(object) => Some(object.as_boa_object()?),
            None => None,
        };
        object.set_prototype(prototype);
        Ok(())
    }

    pub fn as_value(&self) -> &Value<'js> {
        &self.0
    }

    pub fn into_value(self) -> Value<'js> {
        self.0
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        self.0.ctx()
    }

    pub(crate) fn as_boa_object(&self) -> Result<JsObject> {
        self.0
            .value
            .as_object()
            .ok_or_else(|| Error::new_from_js(self.0.type_name(), "object"))
    }
}

pub trait IntoPropKey {
    fn into_prop_key(self) -> PropertyKey;
}

impl IntoPropKey for &str {
    fn into_prop_key(self) -> PropertyKey {
        PropertyKey::from(JsString::from(self))
    }
}

impl IntoPropKey for StdString {
    fn into_prop_key(self) -> PropertyKey {
        PropertyKey::from(JsString::from(self))
    }
}

impl IntoPropKey for &StdString {
    fn into_prop_key(self) -> PropertyKey {
        PropertyKey::from(JsString::from(self.as_str()))
    }
}

impl IntoPropKey for usize {
    fn into_prop_key(self) -> PropertyKey {
        PropertyKey::from(self as u32)
    }
}

impl IntoPropKey for u32 {
    fn into_prop_key(self) -> PropertyKey {
        PropertyKey::from(self)
    }
}

impl IntoPropKey for i32 {
    fn into_prop_key(self) -> PropertyKey {
        if self >= 0 {
            PropertyKey::from(self as u32)
        } else {
            PropertyKey::from(JsString::from(self.to_string()))
        }
    }
}

impl IntoPropKey for PredefinedAtom {
    fn into_prop_key(self) -> PropertyKey {
        self.to_property_key()
    }
}

impl IntoPropKey for &PredefinedAtom {
    fn into_prop_key(self) -> PropertyKey {
        (*self).to_property_key()
    }
}

impl<'js> FromJs<'js> for Value<'js> {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(value)
    }
}

impl<'js> FromJs<'js> for Object<'js> {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let type_name = value.type_name();
        value
            .into_object()
            .ok_or_else(|| Error::new_from_js(type_name, "object"))
    }
}

impl<'js> FromJs<'js> for StdString {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        value
            .value
            .as_string()
            .map(|string| string.to_std_string_escaped())
            .ok_or_else(|| Error::new_from_js(value.type_name(), "string"))
    }
}

impl<'js> FromJs<'js> for bool {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        value
            .as_bool()
            .ok_or_else(|| Error::new_from_js(value.type_name(), "bool"))
    }
}

impl<'js> FromJs<'js> for i32 {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        value
            .as_int()
            .ok_or_else(|| Error::new_from_js(value.type_name(), "i32"))
    }
}

impl<'js> FromJs<'js> for u32 {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let number = value
            .as_number()
            .ok_or_else(|| Error::new_from_js(value.type_name(), "u32"))?;
        if number.fract() != 0.0 || number < 0.0 || number > u32::MAX as f64 {
            return Err(Error::new_from_js(value.type_name(), "u32"));
        }
        Ok(number as u32)
    }
}

impl<'js> FromJs<'js> for f64 {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        value
            .as_float()
            .ok_or_else(|| Error::new_from_js(value.type_name(), "f64"))
    }
}

impl<'js, T> FromJs<'js> for Option<T>
where
    T: FromJs<'js>,
{
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.type_of().is_void() {
            Ok(None)
        } else {
            T::from_js(ctx, value).map(Some)
        }
    }
}

impl<'js> FromJs<'js> for () {
    fn from_js(_: &Ctx<'js>, _: Value<'js>) -> Result<Self> {
        Ok(())
    }
}

impl<'js> IntoJs<'js> for Value<'js> {
    fn into_js(self, _: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self)
    }
}

impl<'js> IntoJs<'js> for &Value<'js> {
    fn into_js(self, _: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.clone())
    }
}

impl<'js> IntoJs<'js> for Object<'js> {
    fn into_js(self, _: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value())
    }
}

impl<'js> IntoJs<'js> for &Object<'js> {
    fn into_js(self, _: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.as_value().clone())
    }
}

impl<'js> IntoJs<'js> for StdString {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.as_str().into_js(ctx)
    }
}

impl<'js> IntoJs<'js> for &StdString {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.as_str().into_js(ctx)
    }
}

impl<'js> IntoJs<'js> for &str {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Value::from_boa(ctx.clone(), JsString::from(self).into()))
    }
}

impl<'js> IntoJs<'js> for bool {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Value::new_bool(ctx.clone(), self))
    }
}

impl<'js> IntoJs<'js> for i32 {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self))
    }
}

impl<'js> IntoJs<'js> for u32 {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        if self <= i32::MAX as u32 {
            Ok(Value::new_int(ctx.clone(), self as i32))
        } else {
            Ok(Value::new_float(ctx.clone(), self as f64))
        }
    }
}

impl<'js> IntoJs<'js> for f64 {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Value::new_float(ctx.clone(), self))
    }
}

impl<'js> IntoJs<'js> for () {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Value::new_undefined(ctx.clone()))
    }
}

impl<'js, T> IntoJs<'js> for Option<T>
where
    T: IntoJs<'js>,
{
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        match self {
            Some(value) => value.into_js(ctx),
            None => Ok(Value::new_undefined(ctx.clone())),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Undefined;

impl Undefined {
    pub fn into_value<'js>(self, ctx: Ctx<'js>) -> Value<'js> {
        Value::new_undefined(ctx)
    }
}

impl<'js> FromJs<'js> for Undefined {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.is_undefined() {
            Ok(Self)
        } else {
            Err(Error::new_from_js(value.type_name(), "undefined"))
        }
    }
}

impl<'js> IntoJs<'js> for Undefined {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value(ctx.clone()))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Null;

impl Null {
    pub fn into_value<'js>(self, ctx: Ctx<'js>) -> Value<'js> {
        Value::new_null(ctx)
    }
}

impl<'js> FromJs<'js> for Null {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.is_null() {
            Ok(Self)
        } else {
            Err(Error::new_from_js(value.type_name(), "null"))
        }
    }
}

impl<'js> IntoJs<'js> for Null {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value(ctx.clone()))
    }
}

#[allow(dead_code)]
struct LifetimeMarker<'js>(PhantomData<&'js ()>);
