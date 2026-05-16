use js_sys::Error as JsError;
use wasm_bindgen::prelude::*;

pub(crate) fn to_js_error(message: impl Into<String>) -> JsValue {
    JsError::new(&message.into()).into()
}
