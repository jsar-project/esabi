use crate::{
    boa_backend::{
        value::{IntoPropKey, IntoJs, Object, Undefined},
    },
    Ctx, Result,
};
use boa_engine::property::PropertyDescriptor;

impl<'js> Object<'js> {
    pub fn prop<K, V>(&self, key: K, prop: V) -> Result<()>
    where
        K: IntoPropKey,
        V: AsProperty<'js>,
    {
        let key = key.into_prop_key();
        let object = self.as_boa_object()?;
        let desc = prop.config(self.ctx())?;
        self.ctx()
            .with_boa(|boa| object.define_property_or_throw(key, desc, boa))
            .map_err(|err| self.ctx().store_exception(err))?;
        Ok(())
    }
}

pub trait AsProperty<'js> {
    fn config(self, ctx: &Ctx<'js>) -> Result<PropertyDescriptor>;
}

#[derive(Debug, Clone, Copy)]
pub struct Property<T> {
    configurable: bool,
    enumerable: bool,
    writable: bool,
    value: T,
}

impl<T> From<T> for Property<T> {
    fn from(value: T) -> Self {
        Self {
            configurable: false,
            enumerable: false,
            writable: false,
            value,
        }
    }
}

impl<T> Property<T> {
    pub fn configurable(mut self) -> Self {
        self.configurable = true;
        self
    }

    pub fn enumerable(mut self) -> Self {
        self.enumerable = true;
        self
    }

    pub fn writable(mut self) -> Self {
        self.writable = true;
        self
    }
}

impl<'js, T> AsProperty<'js> for Property<T>
where
    T: IntoJs<'js>,
{
    fn config(self, ctx: &Ctx<'js>) -> Result<PropertyDescriptor> {
        Ok(PropertyDescriptor::builder()
            .value(self.value.into_js(ctx)?.into_inner())
            .configurable(self.configurable)
            .enumerable(self.enumerable)
            .writable(self.writable)
            .build())
    }
}

impl<'js, T> AsProperty<'js> for T
where
    T: IntoJs<'js>,
{
    fn config(self, ctx: &Ctx<'js>) -> Result<PropertyDescriptor> {
        Ok(PropertyDescriptor::builder()
            .value(self.into_js(ctx)?.into_inner())
            .build())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Accessor<G, S> {
    configurable: bool,
    enumerable: bool,
    get: G,
    set: S,
}

impl<G> Accessor<G, Undefined> {
    pub fn new_get(get: G) -> Self {
        Self {
            configurable: false,
            enumerable: false,
            get,
            set: Undefined,
        }
    }

    pub fn set<S>(self, set: S) -> Accessor<G, S> {
        Accessor {
            configurable: self.configurable,
            enumerable: self.enumerable,
            get: self.get,
            set,
        }
    }
}

impl<S> Accessor<Undefined, S> {
    pub fn new_set(set: S) -> Self {
        Self {
            configurable: false,
            enumerable: false,
            get: Undefined,
            set,
        }
    }

    pub fn get<G>(self, get: G) -> Accessor<G, S> {
        Accessor {
            configurable: self.configurable,
            enumerable: self.enumerable,
            get,
            set: self.set,
        }
    }
}

impl<G, S> Accessor<G, S> {
    pub fn new(get: G, set: S) -> Self {
        Self {
            configurable: false,
            enumerable: false,
            get,
            set,
        }
    }

    pub fn configurable(mut self) -> Self {
        self.configurable = true;
        self
    }

    pub fn enumerable(mut self) -> Self {
        self.enumerable = true;
        self
    }
}

impl<'js, G, S> AsProperty<'js> for Accessor<G, S>
where
    G: IntoJs<'js>,
    S: IntoJs<'js>,
{
    fn config(self, ctx: &Ctx<'js>) -> Result<PropertyDescriptor> {
        Ok(PropertyDescriptor::builder()
            .maybe_get(Some(self.get.into_js(ctx)?.into_inner()))
            .maybe_set(Some(self.set.into_js(ctx)?.into_inner()))
            .configurable(self.configurable)
            .enumerable(self.enumerable)
            .build())
    }
}
