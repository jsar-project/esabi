#![cfg(feature = "engine-boa")]

use esabi::{ArrayBuffer, Context, Runtime, TypedArray};

#[test]
fn boa_array_buffer_and_typed_array_smoke() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        let bytes = TypedArray::<u8>::new_copy(ctx.clone(), [1u8, 2, 3, 4]).unwrap();
        ctx.globals().set("bytes", bytes.clone()).unwrap();

        let result: String = ctx
            .eval("bytes[1] = 9; `${bytes.length}|${bytes.buffer.byteLength}|${bytes[2]}`")
            .unwrap();
        assert_eq!(result, "4|4|3");

        let from_js: TypedArray<u8> = ctx.eval("new Uint8Array([5, 6, 7])").unwrap();
        let from_js_slice: &[u8] = AsRef::<[u8]>::as_ref(&from_js);
        assert_eq!(from_js_slice, &[5, 6, 7]);

        let buffer: ArrayBuffer = ctx.eval("new Uint16Array([0x1234, 0xabcd]).buffer").unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(
            buffer.as_bytes().unwrap(),
            &[0x1234u16.to_ne_bytes(), 0xabcdu16.to_ne_bytes()].concat()
        );

        let typed = TypedArray::<u16>::from_arraybuffer(buffer.clone()).unwrap();
        let typed_slice: &[u16] = AsRef::<[u16]>::as_ref(&typed);
        assert_eq!(typed_slice, &[0x1234, 0xabcd]);
    });
}

#[test]
fn boa_array_buffer_object_checks_and_detach() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        let bytes = TypedArray::<u8>::new_copy(ctx.clone(), [10u8, 20, 30, 40]).unwrap();
        let object = bytes.as_object();
        assert!(object.is_typed_array::<u8>());
        assert!(!object.is_typed_array::<u16>());

        let buffer = bytes.arraybuffer().unwrap();
        let buffer_object = buffer.as_object();
        assert!(buffer_object.is_array_buffer());

        let mut detached = ArrayBuffer::new_copy(ctx.clone(), [1u8, 2, 3, 4]).unwrap();
        assert_eq!(detached.as_bytes().unwrap(), &[1, 2, 3, 4]);
        detached.detach();
        assert!(detached.as_bytes().is_none());
        assert!(detached.as_raw().is_none());
    });
}

#[test]
fn boa_array_buffer_new_and_typed_array_new_roundtrip() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        let buffer = ArrayBuffer::new(ctx.clone(), vec![0x1111u16, 0x2222, 0x3333]).unwrap();
        ctx.globals().set("buffer", buffer.clone()).unwrap();
        let js_result: String = ctx
            .eval(
                "const view = new Uint16Array(buffer); `${buffer.byteLength}|${view.length}|${view[1]}`",
            )
            .unwrap();
        assert_eq!(js_result, "6|3|8738");

        let typed = TypedArray::<u32>::new(ctx.clone(), vec![7u32, 8, 9]).unwrap();
        ctx.globals().set("typed", typed).unwrap();
        let js_typed_result: String = ctx
            .eval("`${typed.length}|${typed.buffer.byteLength}|${typed[2]}`")
            .unwrap();
        assert_eq!(js_typed_result, "3|12|9");
    });
}
