use alloc::{borrow::ToOwned as _, string::String as StdString};
use core::{error::Error as StdError, fmt};

use boa_engine::{JsError, JsNativeError};

use crate::{
    boa_backend::value::{FromJs, Object, Value},
    Ctx, Error, Result,
};

/// A JavaScript `Error` object on the Boa backend.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Exception<'js>(pub(crate) Object<'js>);

impl<'js> Exception<'js> {
    pub fn into_object(self) -> Object<'js> {
        self.0
    }

    pub fn into_value(self) -> Value<'js> {
        self.0.into_value()
    }

    pub fn as_object(&self) -> &Object<'js> {
        &self.0
    }

    pub fn ctx(&self) -> &Ctx<'js> {
        self.0.ctx()
    }

    pub fn from_object(object: Object<'js>) -> Option<Self> {
        Some(Self(object))
    }

    pub fn from_message(ctx: Ctx<'js>, message: &str) -> Result<Self> {
        Self::from_native(ctx, JsNativeError::error().with_message(message.to_owned()))
    }

    pub fn message(&self) -> Option<StdString> {
        self.0.get::<_, Option<StdString>>("message").ok().flatten()
    }

    pub fn stack(&self) -> Option<StdString> {
        self.0.get::<_, Option<StdString>>("stack").ok().flatten()
    }

    pub fn throw_message(ctx: &Ctx<'js>, message: &str) -> Error {
        let (Ok(error) | Err(error)) = Self::from_message(ctx.clone(), message).map(Self::throw);
        error
    }

    pub fn throw_syntax(ctx: &Ctx<'js>, message: &str) -> Error {
        let (Ok(error) | Err(error)) = Self::from_native(
            ctx.clone(),
            JsNativeError::syntax().with_message(message.to_owned()),
        )
        .map(Self::throw);
        error
    }

    pub fn throw_type(ctx: &Ctx<'js>, message: &str) -> Error {
        let (Ok(error) | Err(error)) = Self::from_native(
            ctx.clone(),
            JsNativeError::typ().with_message(message.to_owned()),
        )
        .map(Self::throw);
        error
    }

    pub fn throw_reference(ctx: &Ctx<'js>, message: &str) -> Error {
        let (Ok(error) | Err(error)) = Self::from_native(
            ctx.clone(),
            JsNativeError::reference().with_message(message.to_owned()),
        )
        .map(Self::throw);
        error
    }

    pub fn throw_range(ctx: &Ctx<'js>, message: &str) -> Error {
        let (Ok(error) | Err(error)) = Self::from_native(
            ctx.clone(),
            JsNativeError::range().with_message(message.to_owned()),
        )
        .map(Self::throw);
        error
    }

    pub fn throw_internal(ctx: &Ctx<'js>, message: &str) -> Error {
        let (Ok(error) | Err(error)) = Self::from_native(
            ctx.clone(),
            JsNativeError::error().with_message(message.to_owned()),
        )
                .map(Self::throw);
        error
    }

    pub fn throw(self) -> Error {
        let ctx = self.ctx().clone();
        ctx.throw(self.into_value())
    }

    fn from_native(ctx: Ctx<'js>, error: JsNativeError) -> Result<Self> {
        let value = ctx.with_boa(|boa| JsError::from_native(error).to_opaque(boa));
        let object = Value::from_boa(ctx, value)
            .into_object()
            .ok_or_else(|| Error::new_from_js("value", "object"))?;
        Ok(Self(object))
    }
}

impl<'js> FromJs<'js> for Exception<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let type_name = value.type_name();
        let object = value
            .into_object()
            .ok_or_else(|| Error::new_from_js(type_name, "Exception"))?;
        Ok(Self(object))
    }
}

impl StdError for Exception<'_> {}

impl fmt::Display for Exception<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Error:")?;
        if let Some(message) = self.message() {
            if !message.is_empty() {
                write!(f, " {message}")?;
            }
        }
        if let Some(stack) = self.stack() {
            if !stack.is_empty() {
                write!(f, "\n{stack}")?;
            }
        }
        Ok(())
    }
}
