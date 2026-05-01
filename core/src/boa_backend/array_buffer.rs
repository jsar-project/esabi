use alloc::vec::Vec;
use core::{
    fmt,
    mem::{self, size_of},
    ops::Deref,
    ptr::NonNull,
    result::Result as StdResult,
    slice,
};

use boa_engine::{object::builtins::{AlignedVec, JsArrayBuffer}, JsValue};

use crate::{
    boa_backend::{
        context::Ctx,
        typed_array::TypedArrayItem,
        value::{FromJs, IntoJs, Object, Value},
    },
    Error, JsLifetime, Result,
};

pub struct RawArrayBuffer {
    pub len: usize,
    pub ptr: NonNull<u8>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AsSliceError {
    BufferUsed,
    InvalidAlignment,
}

impl fmt::Display for AsSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsSliceError::BufferUsed => f.write_str("Buffer was already used"),
            AsSliceError::InvalidAlignment => {
                f.write_str("Buffer had a different alignment than was requested")
            }
        }
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ArrayBuffer<'js>(pub(crate) Object<'js>);

unsafe impl<'js> JsLifetime<'js> for ArrayBuffer<'js> {
    type Changed<'to> = ArrayBuffer<'to>;
}

impl<'js> ArrayBuffer<'js> {
    pub fn new<T: Copy>(ctx: Ctx<'js>, src: impl Into<Vec<T>>) -> Result<Self> {
        let src = src.into();
        let bytes = unsafe {
            slice::from_raw_parts(src.as_ptr().cast::<u8>(), src.len() * size_of::<T>())
        };
        Self::from_bytes(ctx, bytes)
    }

    pub fn new_copy<T: Copy>(ctx: Ctx<'js>, src: impl AsRef<[T]>) -> Result<Self> {
        let src = src.as_ref();
        let bytes = unsafe { slice::from_raw_parts(src.as_ptr().cast::<u8>(), mem::size_of_val(src)) };
        Self::from_bytes(ctx, bytes)
    }

    fn from_bytes(ctx: Ctx<'js>, bytes: &[u8]) -> Result<Self> {
        let block = AlignedVec::from_iter(0, bytes.iter().copied());
        let buffer = ctx
            .with_boa(|boa| JsArrayBuffer::from_byte_block(block, boa))
            .map_err(|err| ctx.store_exception(err))?;
        Ok(Self(Object::from_boa_object(ctx, buffer.into())))
    }

    pub fn len(&self) -> usize {
        self.as_boa().map(|buffer| buffer.byte_length()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        let raw = self.as_raw()?;
        Some(unsafe { slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) })
    }

    pub fn as_slice<T: TypedArrayItem>(&self) -> StdResult<&[T], AsSliceError> {
        let raw = self.as_raw().ok_or(AsSliceError::BufferUsed)?;
        if raw.ptr.as_ptr().align_offset(mem::align_of::<T>()) != 0 {
            return Err(AsSliceError::InvalidAlignment);
        }
        let len = raw.len / size_of::<T>();
        Ok(unsafe { slice::from_raw_parts(raw.ptr.as_ptr().cast::<T>(), len) })
    }

    pub fn detach(&mut self) {
        let _ = self.as_boa().and_then(|buffer| buffer.detach(&JsValue::undefined()).ok());
    }

    #[inline]
    pub fn as_value(&self) -> &Value<'js> {
        self.0.as_value()
    }

    #[inline]
    pub fn into_value(self) -> Value<'js> {
        self.0.into_value()
    }

    pub fn from_value(value: Value<'js>) -> Option<Self> {
        Self::from_object(value.into_object()?)
    }

    #[inline]
    pub fn as_object(&self) -> &Object<'js> {
        &self.0
    }

    #[inline]
    pub fn into_object(self) -> Object<'js> {
        self.0
    }

    pub fn from_object(object: Object<'js>) -> Option<Self> {
        Self::from_boa_object(object.as_boa_object().ok()?, object)
    }

    pub fn as_raw(&self) -> Option<RawArrayBuffer> {
        let buffer = self.as_boa()?;
        let data = buffer.data()?;
        let len = data.len();
        let ptr = NonNull::new(data.as_ptr() as *mut u8)?;
        Some(RawArrayBuffer { len, ptr })
    }

    fn as_boa(&self) -> Option<JsArrayBuffer> {
        JsArrayBuffer::from_object(self.0.as_boa_object().ok()?).ok()
    }

    fn from_boa_object(boa: boa_engine::JsObject, object: Object<'js>) -> Option<Self> {
        JsArrayBuffer::from_object(boa).ok()?;
        Some(Self(object))
    }
}

impl<'js, T: TypedArrayItem> AsRef<[T]> for ArrayBuffer<'js> {
    fn as_ref(&self) -> &[T] {
        self.as_slice().expect("ArrayBuffer was detached")
    }
}

impl<'js> Deref for ArrayBuffer<'js> {
    type Target = Object<'js>;

    fn deref(&self) -> &Self::Target {
        self.as_object()
    }
}

impl<'js> AsRef<Object<'js>> for ArrayBuffer<'js> {
    fn as_ref(&self) -> &Object<'js> {
        self.as_object()
    }
}

impl<'js> AsRef<Value<'js>> for ArrayBuffer<'js> {
    fn as_ref(&self) -> &Value<'js> {
        self.as_value()
    }
}

impl<'js> FromJs<'js> for ArrayBuffer<'js> {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let ty_name = value.type_name();
        Self::from_value(value).ok_or_else(|| Error::new_from_js(ty_name, "ArrayBuffer"))
    }
}

impl<'js> IntoJs<'js> for ArrayBuffer<'js> {
    fn into_js(self, _: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(self.into_value())
    }
}

impl<'js> Object<'js> {
    pub fn is_array_buffer(&self) -> bool {
        self.as_boa_object()
            .ok()
            .and_then(|object| JsArrayBuffer::from_object(object).ok())
            .is_some()
    }

    pub unsafe fn ref_array_buffer(&self) -> &ArrayBuffer<'js> {
        mem::transmute(self)
    }

    pub fn as_array_buffer(&self) -> Option<&ArrayBuffer<'js>> {
        self.is_array_buffer()
            .then_some(unsafe { self.ref_array_buffer() })
    }
}
