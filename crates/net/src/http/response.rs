use wasm_bindgen::{JsCast, UnwrapThrowExt};

use crate::js_to_error;

use super::{body::Body, headers::headers_to_js};

/// ResponseOptions "caches" the data used to initialize a [`web_sys::Response].
///
/// It implements [`From<ResponseOptions>`] to allow for easy conversion to [`web_sys::ResponseInit`].
#[derive(Debug)]
struct ResponseOptions {
    pub(crate) headers: http::HeaderMap,
    pub(crate) status_code: http::StatusCode,
    pub(crate) status_text: String,
}

impl ResponseOptions {
    /// Creates a new [`web_sys::ResponseInit`] from a [`ResponseOptions`].
    pub fn into_raw(self) -> web_sys::ResponseInit {
        let mut init = web_sys::ResponseInit::new();

        init.headers(&headers_to_js(&self.headers));
        init.status(self.status_code.as_u16());
        init.status_text(&self.status_text);

        init
    }

    /// Creates a new [`ResponseOptions`] from a [`web_sys::ResponseInit`].
    ///
    /// # Note
    ///
    /// This is possible just by using a reference to the `raw` value.
    pub fn from_raw(init: &web_sys::ResponseInit) -> Self {
        // To get the values out of the ResponseInit, we need to use the `Reflect` API.
        // Note: be careful to handle the case where the value is `undefined` or `null`.

        // Get web_sys::Headers from ResponseInit.
        let headers = match js_sys::Reflect::get(init, &"headers".into()) {
            Ok(headers) => {
                let headers: web_sys::Headers = headers.unchecked_into();
                headers
            }
            Err(_) => web_sys::Headers::new().unwrap_throw(),
        };

        // Get status code from ResponseInit.
        let status_code = match js_sys::Reflect::get(init, &"status".into()) {
            Ok(status_code) => status_code.as_f64().unwrap_or(0.0) as u16,
            Err(_) => 0,
        };

        // Get status text from ResponseInit.
        let status_text = match js_sys::Reflect::get(init, &"statusText".into()) {
            Ok(status_text) => status_text.as_string().unwrap_or_default(),
            Err(_) => String::new(),
        };

        Self {
            headers: crate::http::headers::headers_from_js(&headers),
            status_code: http::StatusCode::from_u16(status_code).unwrap_or_default(),
            status_text,
        }
    }
}

impl From<ResponseOptions> for web_sys::ResponseInit {
    fn from(value: ResponseOptions) -> Self {
        value.into_raw()
    }
}

impl From<web_sys::ResponseInit> for ResponseOptions {
    fn from(value: web_sys::ResponseInit) -> Self {
        Self::from_raw(&value)
    }
}

/// A convenient builder for [`Response`].
#[derive(Debug)]
pub struct ResponseBuilder {
    body: Option<Body>,
    init: ResponseOptions,
}

impl ResponseBuilder {
    /// Creates a new [`ResponseBuilder`] with a [`http::StatusCode`].
    #[inline]
    pub fn new(status_code: http::StatusCode) -> Self {
        Self {
            body: None,
            init: ResponseOptions {
                headers: http::HeaderMap::new(),
                status_code,
                status_text: String::new(),
            },
        }
    }

    /// Sets the [`http::StatusCode`] of the [`Response`].
    #[inline]
    pub fn status(mut self, status_code: http::StatusCode) -> Self {
        self.init.status_code = status_code;
        self
    }

    /// Sets the status text of the [`Response`].
    #[inline]
    pub fn status_text(mut self, status_text: impl Into<String>) -> Self {
        self.init.status_text = status_text.into();
        self
    }

    /// Sets the [`http::HeaderMap`] of the [`Response`].
    #[inline]
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.init.headers = headers;
        self
    }

    /// Sets the [`Body`] of the [`Response`].
    #[inline]
    pub fn body(mut self, body: Body) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets the [`Body`] of the [`Response`] to a string.
    #[inline]
    pub fn text(mut self, text: String) -> Self {
        self.body = Some(Body::Text(text));
        self
    }

    /// Sets the [`Body`] of the [`Response`] to a readable stream.
    #[inline]
    pub fn readable_stream(mut self, stream: web_sys::ReadableStream) -> Self {
        self.body = Some(Body::ReadableStream(stream));
        self
    }

    /// Build the [`Response`] with a JSON body.
    ///
    /// Requires the `json` feature.
    ///
    /// # Note
    ///
    /// This will set the `Content-Type` header to `application/json`.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[inline]
    pub fn json(mut self, json: &impl serde::Serialize) -> Self {
        self.body = Some(Body::Text(serde_json::to_string(json).unwrap()));
        self.init.headers.insert(
            http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        self
    }

    /// Consumes the [`ResponseBuilder`] and returns a [`Response`].
    #[inline]
    pub fn build(self) -> crate::Result<Response> {
        let init: web_sys::ResponseInit = self.init.into();

        match self.body {
            Some(body) => match body {
                Body::Text(body) => {
                    web_sys::Response::new_with_opt_str_and_init(Some(&body), &init)
                }
                Body::ReadableStream(stream) => {
                    web_sys::Response::new_with_opt_readable_stream_and_init(Some(&stream), &init)
                }
            },
            None => web_sys::Response::new_with_opt_readable_stream_and_init(None, &init),
        }
        .map_err(js_to_error)
        .map(Response::from_raw)
    }
}

/// A wrapper around [`web_sys::Response`].
#[derive(Debug)]
pub struct Response {
    raw: web_sys::Response,
    // Cached Rust representations of the request inners.
    init: ResponseOptions,
}

impl Response {
    /// Creates a new [`Response`] from a [`web_sys::Response`].
    #[inline]
    pub fn from_raw(response: web_sys::Response) -> Self {
        let init = ResponseOptions::from_raw(response.unchecked_ref());

        Self {
            raw: response,
            init,
        }
    }

    /// Creates a new [`web_sys::Response`] from a [`Response`].
    #[inline]
    pub fn into_raw(self) -> web_sys::Response {
        self.raw
    }

    /// Creates a new [`ResponseBuilder`] with a [`http::StatusCode`].
    #[inline]
    pub fn builder(status_code: http::StatusCode) -> ResponseBuilder {
        ResponseBuilder::new(status_code)
    }

    /// Returns the [`http::HeaderMap`] of the [`Response`].
    #[inline]
    pub fn headers(&self) -> &http::HeaderMap {
        &self.init.headers
    }

    /// Returns the [`http::StatusCode`] of the [`Response`].
    #[inline]
    pub fn status_code(&self) -> http::StatusCode {
        self.init.status_code
    }

    /// Returns the status text of the [`Response`].
    #[inline]
    pub fn status_text(&self) -> &str {
        &self.init.status_text
    }

    /// Extracts the [`Body`] from the [`Response`].
    #[inline]
    pub fn body(self) -> Body {
        Body::from(self.raw.body().unwrap_throw())
    }

    /// Extracts the [`Body`] from the [`Response`] as a string.
    #[inline]
    pub async fn text(self) -> crate::Result<String> {
        let promise = self.raw.text().map_err(js_to_error)?;

        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map(|value| value.as_string().unwrap_throw())
            .map_err(js_to_error)
    }

    /// Extracts the [`Body`] from the [`Response`] as a JSON value.
    ///
    /// Requires the `json` feature.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[inline]
    pub async fn json(self) -> crate::Result<serde_json::Value> {
        use gloo_utils::format::JsValueSerdeExt;

        let promise = self.raw.json().map_err(js_to_error)?;

        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map(|value| value.into_serde().unwrap_throw())
            .map_err(js_to_error)
    }
}
