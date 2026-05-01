use alloc::vec::Vec;
use core::{
    fmt,
    marker::PhantomData,
    mem::{self, size_of},
    ops::Deref,
    ptr::NonNull,
    slice,
};

use boa_engine::{
    builtins::typed_array::TypedArrayKind,
    object::builtins::{
        JsArrayBuffer, JsBigInt64Array, JsBigUint64Array, JsFloat32Array, JsFloat64Array,
        JsInt16Array, JsInt32Array, JsInt8Array, JsTypedArray, JsUint16Array, JsUint32Array,
        JsUint8Array,
    },
};

use crate::{
    boa_backend::{
        array_buffer::{ArrayBuffer, RawArrayBuffer},
        context::Ctx,
        value::{FromJs, IntoJs, Object, Value},
    },
    Error, JsLifetime, Result,
};

pub trait TypedArrayItem: Copy {
    const CLASS_NAME: &'static str;
    const KIND: TypedArrayKind;

    fn from_bytes(bytes: &[u8], ctx: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::JsValue>;
    fn from_array_buffer(
        buffer: JsArrayBuffer,
        ctx: &mut boa_engine::Context,
    ) -> boa_engine::JsResult<boa_engine::JsValue>;
}

macro_rules! typed_array_items {
    ($($name:literal => $type:ty, $kind:expr, $wrapper:ident,)*) => {
        $(
            impl TypedArrayItem for $type {
                const CLASS_NAME: &'static str = $name;
                const KIND: TypedArrayKind = $kind;

                fn from_bytes(
                    bytes: &[u8],
                    ctx: &mut boa_engine::Context,
                ) -> boa_engine::JsResult<boa_engine::JsValue> {
                    let values = bytes
                        .chunks_exact(size_of::<$type>())
                        .map(|chunk| <$type>::from_ne_bytes(chunk.try_into().expect("fixed-width chunk")));
                    $wrapper::from_iter(values, ctx).map(Into::into)
                }

                fn from_array_buffer(
                    buffer: JsArrayBuffer,
                    ctx: &mut boa_engine::Context,
                ) -> boa_engine::JsResult<boa_engine::JsValue> {
                    $wrapper::from_array_buffer(buffer, ctx).map(Into::into)
                }
            }
        )*
    };
}

typed_array_items! {
    "Int8Array" => i8, TypedArrayKind::Int8, JsInt8Array,
    "Uint8Array" => u8, TypedArrayKind::Uint8, JsUint8Array,
    "Int16Array" => i16, TypedArrayKind::Int16, JsInt16Array,
    "Uint16Array" => u16, TypedArrayKind::Uint16, JsUint16Array,
    "Int32Array" => i32, TypedArrayKind::Int32, JsInt32Array,
    "Uint32Array" => u32, TypedArrayKind::Uint32, JsUint32Array,
    "Float32Array" => f32, TypedArrayKind::Float32, JsFloat32Array,
    "Float64Array" => f64, TypedArrayKind::Float64, JsFloat64Array,
    "BigInt64Array" => i64, TypedArrayKind::BigInt64, JsBigInt64Array,
    "BigUint64Array" => u64, TypedArrayKind::BigUint64, JsBigUint64Array,
}

#[repr(transparent)]
pub struct TypedArray<'js, T>(pub(crate) Object<'js>, PhantomData<T>);

unsafe impl<'js, T: JsLifetime<'js>> JsLifetime<'js> for TypedArray<'js, T> {
    type Changed<'to> = TypedArray<'to, T::Changed<'to>>;
}

impl<'js, T> fmt::Debug for TypedArray<'js, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("TypedArray").field(&self.0).finish()
    }
}

impl<'js, T> Clone for TypedArray<'js, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<'js, T> TypedArray<'js, T> {
    pub fn new(ctx: Ctx<'js>, src: impl Into<Vec<T>>) -> Result<Self>
    where
        T: TypedArrayItem,
    {
        let src = src.into();
        let bytes = unsafe {
            slice::from_raw_parts(src.as_ptr().cast::<u8>(), src.len() * size_of::<T>())
        };
        Self::from_bytes(ctx, bytes)
    }

    pub fn new_copy(ctx: Ctx<'js>, src: impl AsRef<[T]>) -> Result<Self>
    where
        T: TypedArrayItem,
    {
        let src = src.as_ref();
        let bytes = unsafe { slice::from_raw_parts(src.as_ptr().cast::<u8>(), mem::size_of_val(src)) };
        Self::from_bytes(ctx, bytes)
    }

    fn from_bytes(ctx: Ctx<'js>, bytes: &[u8]) -> Result<Self>
    where
        T: TypedArrayItem,
    {
        let value = ctx
            .with_boa(|boa| T::from_bytes(bytes, boa))
            .map_err(|err| ctx.store_exception(err))?;
        Ok(Self(
            Object::from_boa_object(
                ctx.clone(),
                value.as_object().expect("typed array constructor returns object"),
            ),
            PhantomData,
        ))
    }

    pub fn len(&self) -> usize
    where
        T: TypedArrayItem,
    {
        self.as_boa()
            .and_then(|typed| self.ctx().with_boa(|boa| typed.length(boa).ok()))
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool
    where
        T: TypedArrayItem,
    {
        self.len() == 0
    }

    #[inline]
    pub fn as_value(&self) -> &Value<'js> {
        self.0.as_value()
    }

    #[inline]
    pub fn into_value(self) -> Value<'js> {
        self.0.into_value()
    }

    pub fn from_value(value: Value<'js>) -> Result<Self>
    where
        T: TypedArrayItem,
    {
        let type_name = value.type_name();
        Self::from_object(
            value
                .into_object()
                .ok_or_else(|| Error::new_from_js(type_name, "object"))?,
        )
    }

    #[inline]
    pub fn as_object(&self) -> &Object<'js> {
        &self.0
    }

    #[inline]
    pub fn into_object(self) -> Object<'js> {
        self.0
    }

    pub fn from_object(object: Object<'js>) -> Result<Self>
    where
        T: TypedArrayItem,
    {
        let is_kind = object
            .as_boa_object()
            .ok()
            .and_then(|object| JsTypedArray::from_object(object).ok())
            .and_then(|typed| typed.kind())
            .map(|kind| kind == T::KIND)
            .unwrap_or(false);
        if is_kind {
            Ok(Self(object, PhantomData))
        } else {
            Err(Error::new_from_js("object", T::CLASS_NAME))
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]>
    where
        T: TypedArrayItem,
    {
        let (_, len, ptr) = Self::get_raw_bytes(self.as_value())?;
        Some(unsafe { slice::from_raw_parts(ptr.as_ptr(), len) })
    }

    pub fn as_raw(&self) -> Option<RawArrayBuffer>
    where
        T: TypedArrayItem,
    {
        let (_, len, ptr) = Self::get_raw_bytes(self.as_value())?;
        Some(RawArrayBuffer { len, ptr })
    }

    pub fn arraybuffer(&self) -> Result<ArrayBuffer<'js>>
    where
        T: TypedArrayItem,
    {
        let typed = self
            .as_boa()
            .ok_or_else(|| Error::new_from_js("object", T::CLASS_NAME))?;
        let buffer = self
            .ctx()
            .with_boa(|boa| typed.buffer(boa))
            .map_err(|err| self.ctx().store_exception(err))?;
        let object = buffer
            .as_object()
            .ok_or_else(|| Error::new_from_js("value", "ArrayBuffer"))?;
        let value = Value::from_boa(self.ctx().clone(), object.into());
        ArrayBuffer::from_js(self.ctx(), value)
    }

    pub fn from_arraybuffer(arraybuffer: ArrayBuffer<'js>) -> Result<Self>
    where
        T: TypedArrayItem,
    {
        let ctx = arraybuffer.ctx().clone();
        let boa_buffer = JsArrayBuffer::from_object(arraybuffer.as_object().as_boa_object()?)
            .map_err(|err| ctx.store_exception(err))?;
        let value = ctx
            .with_boa(|boa| T::from_array_buffer(boa_buffer, boa))
            .map_err(|err| ctx.store_exception(err))?;
        Ok(Self(
            Object::from_boa_object(
                ctx,
                value.as_object().expect("typed array constructor returns object"),
            ),
            PhantomData,
        ))
    }

    pub(crate) fn get_raw_bytes(val: &Value<'js>) -> Option<(usize, usize, NonNull<u8>)>
    where
        T: TypedArrayItem,
    {
        let object = val.as_object()?;
        let typed = JsTypedArray::from_object(object.as_boa_object().ok()?).ok()?;
        if typed.kind()? != T::KIND {
            return None;
        }
        let byte_offset = val.ctx().with_boa(|boa| typed.byte_offset(boa).ok())?;
        let byte_length = val.ctx().with_boa(|boa| typed.byte_length(boa).ok())?;
        let buffer = val
            .ctx()
            .with_boa(|boa| typed.buffer(boa).ok())
            .and_then(|value| value.as_object())?;
        let buffer = JsArrayBuffer::from_object(buffer).ok()?;
        let data = buffer.data()?;
        let byte_offset = byte_offset as usize;
        let byte_length = byte_length as usize;
        let end = byte_offset.checked_add(byte_length)?;
        if end > data.len() {
            return None;
        }
        let ptr = NonNull::new(unsafe { data.as_ptr().add(byte_offset) as *mut u8 })?;
        Some((size_of::<T>(), byte_length, ptr))
    }

    pub(crate) fn get_raw(val: &Value<'js>) -> Option<(usize, NonNull<T>)>
    where
        T: TypedArrayItem,
    {
        let (step, len, ptr) = Self::get_raw_bytes(val)?;
        if step != size_of::<T>() || len % size_of::<T>() != 0 {
            return None;
        }
        debug_assert_eq!(ptr.as_ptr().align_offset(mem::align_of::<T>()), 0);
        Some((len / size_of::<T>(), ptr.cast::<T>()))
    }

    fn as_boa(&self) -> Option<JsTypedArray> {
        JsTypedArray::from_object(self.0.as_boa_object().ok()?).ok()
    }
}

impl<'js, T: TypedArrayItem> AsRef<[T]> for TypedArray<'js, T> {
    fn as_ref(&self) -> &[T] {
        let (len, ptr) =
            Self::get_raw(self.as_value()).unwrap_or_else(|| panic!("{}", T::CLASS_NAME));
        unsafe { slice::from_raw_parts(ptr.as_ptr(), len) }
    }
}

impl<'js, T> Deref for TypedArray<'js, T> {
    type Target = Object<'js>;

    fn deref(&self) -> &Self::Target {
        self.as_object()
    }
}

impl<'js, T> AsRef<Object<'js>> for TypedArray<'js, T> {
    fn as_ref(&self) -> &Object<'js> {
        self.as_object()
    }
}

impl<'js, T> AsRef<Value<'js>> for TypedArray<'js, T> {
    fn as_ref(&self) -> &Value<'js> {
        self.as_value()
    }
}

impl<'js, T> TryFrom<TypedArray<'js, T>> for ArrayBuffer<'js>
where
    T: TypedArrayItem,
{
    type Error = Error;

    fn try_from(ta: TypedArray<'js, T>) -> Result<Self> {
        ta.arraybuffer()
    }
}

impl<'js, T> TryFrom<ArrayBuffer<'js>> for TypedArray<'js, T>
where
    T: TypedArrayItem,
{
    type Error = Error;

    fn try_from(ab: ArrayBuffer<'js>) -> Result<Self> {
        Self::from_arraybuffer(ab)
    }
}

impl<'js, T> FromJs<'js> for TypedArray<'js, T>
where
    T: TypedArrayItem,
{
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Self::from_value(value)
    }
}

impl<'js, T> IntoJs<'js> for TypedArray<'js, T> {
    fn into_js(self, _: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value())
    }
}

impl<'js> Object<'js> {
    pub fn is_typed_array<T: TypedArrayItem>(&self) -> bool {
        self.as_boa_object()
            .ok()
            .and_then(|object| JsTypedArray::from_object(object).ok())
            .and_then(|typed| typed.kind())
            .map(|kind| kind == T::KIND)
            .unwrap_or(false)
    }

    pub unsafe fn ref_typed_array<'a, T: TypedArrayItem>(&'a self) -> &'a TypedArray<'js, T> {
        mem::transmute(self)
    }

    pub fn as_typed_array<T: TypedArrayItem>(&self) -> Option<&TypedArray<'js, T>> {
        self.is_typed_array::<T>()
            .then(|| unsafe { self.ref_typed_array() })
    }
}
