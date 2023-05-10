// Request:
// Any body that you want to add to your request: this can be a Blob,
// an ArrayBuffer, a TypedArray, a DataView, a FormData, a URLSearchParams,
// string object or literal, or a ReadableStream object.

// Response:
// ReadableStream, ArrayBuffer, Blob, FormData, Json, or Text.

use wasm_bindgen::JsValue;
use web_sys::ReadableStream;

#[derive(Debug, Clone)]
pub enum Body {
    Text(String),
    ReadableStream(web_sys::ReadableStream),
    // TODO: Add support for the other types.
}

impl From<ReadableStream> for Body {
    fn from(value: ReadableStream) -> Self {
        Self::ReadableStream(value)
    }
}

impl From<Body> for JsValue {
    fn from(value: Body) -> Self {
        match value {
            Body::Text(text) => text.into(),
            Body::ReadableStream(stream) => stream.into(),
        }
    }
}
