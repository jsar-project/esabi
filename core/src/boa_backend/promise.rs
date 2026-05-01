use boa_engine::builtins::promise::PromiseState as BoaPromiseState;

use crate::{
    boa_backend::value::{FromJs, IntoJs, Object, Type, Value},
    Ctx, Error, Result,
};

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Promise<'js>(pub(crate) Object<'js>);

#[derive(Clone, Debug)]
pub enum PromiseState<'js> {
    Pending,
    Fulfilled(Value<'js>),
    Rejected(Value<'js>),
}

impl<'js> Promise<'js> {
    pub fn from_object(object: Object<'js>) -> Result<Self> {
        Ok(Self(object))
    }

    pub fn state(&self) -> PromiseState<'js> {
        let value = self.0.as_value().clone().into_inner();
        match value.as_promise() {
            Some(promise) => match promise.state() {
                BoaPromiseState::Pending => PromiseState::Pending,
                BoaPromiseState::Fulfilled(value) => {
                    PromiseState::Fulfilled(Value::from_boa(self.ctx().clone(), value.clone()))
                }
                BoaPromiseState::Rejected(value) => {
                    PromiseState::Rejected(Value::from_boa(self.ctx().clone(), value.clone()))
                }
            },
            None => PromiseState::Rejected(Value::new_undefined(self.ctx().clone())),
        }
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
}

impl<'js> FromJs<'js> for Promise<'js> {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.type_of() != Type::Object {
            return Err(Error::new_from_js(value.type_name(), "promise"));
        }
        Ok(Self(
            value
                .into_object()
                .ok_or_else(|| Error::new_from_js("value", "promise"))?,
        ))
    }
}

impl<'js> IntoJs<'js> for Promise<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value())
    }
}
