use std::str::FromStr;

use gloo_utils::iter::UncheckedIter;
use http::{HeaderMap, HeaderName, HeaderValue};
use js_sys::{Array, Map};
use wasm_bindgen::JsCast;

pub(crate) fn headers_to_js(headers: &HeaderMap) -> web_sys::Headers {
    let js_headers = web_sys::Headers::new().unwrap();
    for (name, value) in headers {
        js_headers
            .append(name.as_str(), value.to_str().unwrap())
            .unwrap();
    }
    js_headers
}

pub(crate) fn headers_from_js(headers: web_sys::Headers) -> HeaderMap {
    // Here we cheat and cast to a map even though `self` isn't, because the method names match
    // and everything works. Is there a better way? Should there be a `MapLike` or
    // `MapIterator` type in `js_sys`?
    let fake_map: &Map = headers.unchecked_ref();
    let iter = UncheckedIter::from(fake_map.entries()).map(|entry| {
        let entry: Array = entry.unchecked_into();
        let key = entry.get(0);
        let value = entry.get(1);

        let header_name = HeaderName::from_str(&key.as_string().unwrap()).unwrap();
        let header_value = HeaderValue::from_str(&value.as_string().unwrap()).unwrap();

        (header_name, header_value)
    });

    iter.collect()
}
