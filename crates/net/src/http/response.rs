use super::{
    body::Body,
    headers::{headers_from_js, headers_to_js},
};

/// ResponseInit "caches" the data used to initialize a [`web_sys::Response].
///
/// It implements [`From<ResponseInit>`] to allow for easy conversion to [`web_sys::ResponseInit`].
#[derive(Debug)]
struct ResponseInit {
    pub(crate) headers: http::HeaderMap,
    pub(crate) status_code: http::StatusCode,
    pub(crate) status_text: String,
}

impl From<ResponseInit> for web_sys::ResponseInit {
    fn from(value: ResponseInit) -> Self {
        let mut init = web_sys::ResponseInit::new();

        init.headers(&headers_to_js(&value.headers));
        init.status(value.status_code.as_u16());
        init.status_text(&value.status_text);

        init
    }
}

/// A convenient builder for [`Response`].
#[derive(Debug)]
#[must_use = "ResponseBuilder does nothing unless you call `build`"]
pub struct ResponseBuilder {
    body: Option<Body>,
    init: ResponseInit,
}

/// A wrapper around [`web_sys::Response`].
#[derive(Debug)]
pub struct Response {
    body: Option<Body>,
    init: ResponseInit,
}

impl ResponseBuilder {
    /// Creates a new [`ResponseBuilder`] with a [`http::StatusCode`].
    pub fn new(status_code: http::StatusCode) -> Self {
        Self {
            body: None,
            init: ResponseInit {
                headers: http::HeaderMap::new(),
                status_code,
                status_text: String::new(),
            },
        }
    }

    /// Sets the [`http::StatusCode`] of the [`Response`].
    pub fn status(mut self, status_code: http::StatusCode) -> Self {
        self.init.status_code = status_code;
        self
    }

    /// Sets the status text of the [`Response`].
    pub fn status_text(mut self, status_text: impl Into<String>) -> Self {
        self.init.status_text = status_text.into();
        self
    }

    /// Sets the [`http::HeaderMap`] of the [`Response`].
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.init.headers = headers;
        self
    }

    /// Consumes the [`ResponseBuilder`] and returns a [`Response`].
    pub fn build(self) -> Response {
        Response {
            body: self.body,
            init: self.init,
        }
    }
}

impl Response {
    /// Creates a new [`Response`] from a [`web_sys::Response`].
    pub fn from_raw(response: web_sys::Response) -> Self {
        response.into()
    }

    /// Creates a new [`web_sys::Response`] from a [`Response`].
    pub fn into_raw(self) -> web_sys::Response {
        self.into()
    }

    /// Creates a new [`ResponseBuilder`] with a [`http::StatusCode`].
    pub fn builder(status_code: http::StatusCode) -> ResponseBuilder {
        ResponseBuilder::new(status_code)
    }

    /// Returns the [`Body`] of the [`Response`].
    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    /// Returns the [`http::HeaderMap`] of the [`Response`].
    pub fn headers(&self) -> &http::HeaderMap {
        &self.init.headers
    }

    /// Returns the [`http::StatusCode`] of the [`Response`].
    pub fn status_code(&self) -> http::StatusCode {
        self.init.status_code
    }

    /// Returns the status text of the [`Response`].
    pub fn status_text(&self) -> &str {
        &self.init.status_text
    }
}

impl From<Response> for web_sys::Response {
    fn from(value: Response) -> Self {
        let init: web_sys::ResponseInit = value.init.into();

        match value.body {
            None => web_sys::Response::new_with_opt_readable_stream_and_init(None, &init),
            Some(Body::ReadableStream(stream)) => {
                web_sys::Response::new_with_opt_readable_stream_and_init(Some(&stream), &init)
            }
            Some(Body::Text(string)) => {
                web_sys::Response::new_with_opt_str_and_init(Some(&string), &init)
            }
        }
        .unwrap()
    }
}

impl From<web_sys::Response> for Response {
    fn from(value: web_sys::Response) -> Self {
        Response {
            body: value.body().map(Body::from),
            init: ResponseInit {
                headers: headers_from_js(value.headers()),
                status_code: http::StatusCode::from_u16(value.status()).unwrap(),
                status_text: value.status_text(),
            },
        }
    }
}
