use boa_parity_fixtures::{fixtures, run_source, support_matrix};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn support_matrix_json() -> String {
    serde_json::to_string_pretty(&support_matrix()).expect("support matrix should serialize")
}

#[wasm_bindgen]
pub fn examples_json() -> String {
    serde_json::to_string_pretty(&fixtures()).expect("examples should serialize")
}

#[wasm_bindgen]
pub fn run_playground(source: &str) -> core::result::Result<String, JsValue> {
    serde_json::to_string_pretty(&run_source(source))
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
