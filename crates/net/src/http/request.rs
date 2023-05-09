use std::convert::TryInto;

use http::{header::InvalidHeaderValue, HeaderName, HeaderValue};
use wasm_bindgen::JsCast;
use web_sys::RequestCache;

use crate::{js_to_error, Error};

use super::{
    body::Body,
    headers::{headers_from_js, headers_to_js},
    Response,
};

/// RequestInit caches the data used to initialize a [`web_sys::Request`].
///
/// It implements [`From<RequestInit>`] to allow for easy conversion to [`web_sys::RequestInit`].
#[derive(Debug)]
struct RequestInit {
    method: http::Method,
    headers: http::HeaderMap,
    body: Option<Body>,
    cache: web_sys::RequestCache,
    credentials: web_sys::RequestCredentials,
    integrity: String,
    mode: web_sys::RequestMode,
    redirect: web_sys::RequestRedirect,
    referrer: String,
    referrer_policy: web_sys::ReferrerPolicy,
    // pub(crate) signal: Option<&'a web_sys::AbortSignal>,
}

impl From<RequestInit> for web_sys::RequestInit {
    fn from(value: RequestInit) -> Self {
        let mut init = web_sys::RequestInit::new();

        init.method(value.method.as_str());
        init.headers(&headers_to_js(&value.headers));
        init.body(value.body.map(Into::into).as_ref());
        init.cache(value.cache);
        init.credentials(value.credentials);
        init.integrity(&value.integrity);
        init.mode(value.mode);
        init.redirect(value.redirect);
        init.referrer(&value.referrer);
        init.referrer_policy(value.referrer_policy);

        init
    }
}

/// A convenient builder for [`Request`].
#[derive(Debug)]
#[must_use = "RequestBuilder does nothing unless you assign a body or call `build`"]
pub struct RequestBuilder {
    // url
    url: String,
    init: RequestInit,
}

/// A wrapper around [`web_sys::Request`].
#[derive(Debug)]
pub struct Request {
    url: String,
    init: RequestInit,
}

/// A macro to generate "method" and "try_method" functions for [`RequestBuilder`].
macro_rules! gen_method {
    ($method:ident) => {
        paste::item! {
            #[doc = "Create a new [`RequestBuilder`] from a [`url::Url`]."]
            #[doc = ""]
            #[doc = concat!("# Note\n\nThis function is equivalent to [`RequestBuilder::new(http::Method::", stringify!($name), ", url)`].")]
            #[inline]
            pub fn [<$method:lower>](url: ::std::string::String) -> $crate::http::RequestBuilder {
                $crate::http::RequestBuilder::new(http::Method::[<$method:upper>], url)
            }

            // #[doc = "Tries to create a new [`RequestBuilder`]."]
            // #[doc = ""]
            // #[doc = "# Errors\n\nThis function will return an error if URL parsing fails."]
            // #[doc = ""]
            // #[doc = concat!("# Note\n\nThis function is equivalent to [`RequestBuilder::try_new(http::Method::", stringify!($name), ", url)`].")]
            // #[inline]
            // pub fn [<try_ $method:lower>]<T>(url: T) -> Result<$crate::http::RequestBuilder, T::Error>
            // where
            //     T: ::std::convert::TryInto<::url::Url>,
            // {
            //     $crate::http::RequestBuilder::try_new(http::Method::[<$method:upper>], url)
            // }
        }
    };
}

impl RequestBuilder {
    /// Create a new [`RequestBuilder`] from a [`http::Method`] and a url.
    #[inline]
    pub fn new(method: http::Method, url: String) -> Self {
        Self {
            url,
            init: RequestInit {
                method,
                headers: http::HeaderMap::new(),
                body: None,
                cache: RequestCache::Default,
                credentials: web_sys::RequestCredentials::Omit,
                integrity: String::default(),
                mode: web_sys::RequestMode::Cors,
                redirect: web_sys::RequestRedirect::Follow,
                referrer: String::from("about:client"),
                referrer_policy: web_sys::ReferrerPolicy::None,
            },
        }
    }

    // Tries to create a new [`RequestBuilder`].
    //
    // # Errors
    //
    // This function will return if URL parsing fails.
    // #[inline]
    // pub fn try_new<T>(method: http::Method, url: T) -> Result<Self, T::Error>
    // where
    //     T: TryInto<url::Url>,
    // {
    //     Ok(Self::new(method, url.try_into()?))
    // }

    gen_method!(get);
    gen_method!(post);
    gen_method!(put);
    gen_method!(delete);
    gen_method!(head);
    gen_method!(options);
    gen_method!(connect);
    gen_method!(patch);
    gen_method!(trace);

    /// Set the [`http::HeaderMap`] of the [`Request`].
    #[inline]
    pub fn headers(mut self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>) -> Self {
        self.init.headers = headers.collect();
        self
    }

    /// Sets a single header of the [`Request`].
    ///
    /// # Note
    ///
    /// This will overwrite any existing header with the same name.
    #[inline]
    pub fn header(mut self, name: &HeaderName, value: impl Into<HeaderValue>) -> Self {
        self.init.headers.insert(name, value.into());
        self
    }

    /// Tries to set a single header of the [`Request`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the header key or value is invalid.
    #[inline]
    pub fn try_header(
        mut self,
        name: &HeaderName,
        value: impl TryInto<HeaderValue, Error = InvalidHeaderValue>,
    ) -> Result<Self, InvalidHeaderValue> {
        self.init.headers.insert(name, value.try_into()?);
        Ok(self)
    }

    /// Sets a single header of the [`Request`]. Panics if the header key or value is invalid.
    #[inline]
    pub fn try_header_unchecked(
        mut self,
        name: &HeaderName,
        value: impl TryInto<HeaderValue, Error = InvalidHeaderValue>,
    ) -> Self {
        self.init.headers.insert(name, value.try_into().unwrap());
        self
    }

    // skip body

    /// Set the [`web_sys::RequestCache`] of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/cache)
    ///
    /// # Note
    ///
    /// This is set to [`web_sys::RequestCache::Default`] by default.
    #[inline]
    pub fn cache(mut self, cache: web_sys::RequestCache) -> Self {
        self.init.cache = cache;
        self
    }

    /// Set the [`web_sys::RequestCredentials`] of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials)
    ///
    /// # Note
    ///
    /// This is set to [`web_sys::RequestCredentials::Omit`] by default.
    #[inline]
    pub fn credentials(mut self, credentials: web_sys::RequestCredentials) -> Self {
        self.init.credentials = credentials;
        self
    }

    /// Set the [subresource integrity](https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity) value of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/integrity)
    #[inline]
    pub fn integrity(mut self, integrity: impl Into<String>) -> Self {
        self.init.integrity = integrity.into();
        self
    }

    /// Set the [`web_sys::RequestMode`] of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/mode)
    ///
    /// # Note
    ///
    /// This is set to [`web_sys::RequestMode::Cors`] by default.
    #[inline]
    pub fn mode(mut self, mode: web_sys::RequestMode) -> Self {
        self.init.mode = mode;
        self
    }

    /// Set the [`web_sys::RequestRedirect`] of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/redirect)
    ///
    /// # Note
    ///
    /// This is set to [`web_sys::RequestRedirect::Follow`] by default.
    #[inline]
    pub fn redirect(mut self, redirect: web_sys::RequestRedirect) -> Self {
        self.init.redirect = redirect;
        self
    }

    /// Set the referrer of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/referrer)
    ///
    /// # Note
    ///
    /// This is set to `"about:client"` by default.
    #[inline]
    pub fn referrer(mut self, referrer: impl Into<String>) -> Self {
        self.init.referrer = referrer.into();
        self
    }

    /// Set the [`web_sys::ReferrerPolicy`] of the [`Request`].
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Request/referrerPolicy)
    ///
    /// # Note
    ///
    /// This is set to [`web_sys::ReferrerPolicy::None`] by default.
    #[inline]
    pub fn referrer_policy(mut self, referrer_policy: web_sys::ReferrerPolicy) -> Self {
        self.init.referrer_policy = referrer_policy;
        self
    }

    // TODO: skip signals for now

    /// Build the [`Request`] with an empty body.
    #[inline]
    pub fn build(self) -> Request {
        Request {
            url: self.url,
            init: self.init,
        }
    }

    /// Build the [`Request`] with a body.
    #[inline]
    pub fn body(self, body: impl Into<Body>) -> Request {
        Request {
            url: self.url,
            init: RequestInit {
                body: Some(body.into()),
                ..self.init
            },
        }
    }

    /// Build the [`Request`] with a text body.
    #[inline]
    pub fn text(self, text: impl Into<String>) -> Request {
        self.body(Body::Text(text.into()))
    }

    /// Build the [`Request`] with a JSON body.
    /// Requires the `json` feature.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[inline]
    pub fn json<T: serde::Serialize>(self, value: &T) -> Result<Request, Error> {
        let json = serde_json::to_string(value)?;
        Ok(self.text(json))
    }

    /// Build and Send the [`Request`] using the `fetch` API.
    pub async fn send(self) -> Result<Response, Error> {
        self.build().send().await
    }
}

// Getters into RequestInit
impl Request {
    gen_method!(get);
    gen_method!(post);
    gen_method!(put);
    gen_method!(delete);
    gen_method!(head);
    gen_method!(options);
    gen_method!(connect);
    gen_method!(patch);
    gen_method!(trace);

    /// Get the [`http::Method`] of the [`Request`].
    #[inline]
    pub fn method(&self) -> &http::Method {
        &self.init.method
    }

    /// Get the [`http::HeaderMap`] of the [`Request`].
    #[inline]
    pub fn headers(&self) -> &http::HeaderMap {
        &self.init.headers
    }

    /// Get the [`web_sys::RequestCache`] of the [`Request`].
    #[inline]
    pub fn cache(&self) -> web_sys::RequestCache {
        self.init.cache
    }

    /// Get the [`web_sys::RequestCredentials`] of the [`Request`].
    #[inline]
    pub fn credentials(&self) -> web_sys::RequestCredentials {
        self.init.credentials
    }

    /// Get the [subresource integrity](https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity) value of the [`Request`].
    #[inline]
    pub fn integrity(&self) -> &String {
        &self.init.integrity
    }

    /// Get the [`web_sys::RequestMode`] of the [`Request`].
    #[inline]
    pub fn mode(&self) -> web_sys::RequestMode {
        self.init.mode
    }

    /// Get the [`web_sys::RequestRedirect`] of the [`Request`].
    #[inline]
    pub fn redirect(&self) -> web_sys::RequestRedirect {
        self.init.redirect
    }

    /// Get the referrer of the [`Request`].
    #[inline]
    pub fn referrer(&self) -> &String {
        &self.init.referrer
    }

    /// Get the [`web_sys::ReferrerPolicy`] of the [`Request`].
    #[inline]
    pub fn referrer_policy(&self) -> web_sys::ReferrerPolicy {
        self.init.referrer_policy
    }

    // TODO: skip signals for now

    /// Get the [`url::Url`] of the [`Request`].
    #[inline]
    pub fn url(&self) -> &String {
        &self.url
    }

    /// Sends the [`Request`] using the `fetch` API.
    pub async fn send(self) -> Result<Response, Error> {
        let request = web_sys::Request::new_with_str_and_init(self.url.as_str(), &self.init.into())
            .map_err(js_to_error)?;

        let resp = wasm_bindgen_futures::JsFuture::from(
            web_sys::window().unwrap().fetch_with_request(&request),
        )
        .await
        .map_err(js_to_error)?;

        Ok(Response::from(
            resp.dyn_into::<web_sys::Response>().unwrap(),
        ))
    }
}

impl From<web_sys::Request> for Request {
    fn from(value: web_sys::Request) -> Self {
        Self {
            url: value.url(),
            init: RequestInit {
                method: http::Method::from_bytes(value.method().as_bytes()).unwrap(),
                headers: headers_from_js(value.headers()),
                body: value.body().map(Body::from),
                cache: value.cache(),
                credentials: value.credentials(),
                integrity: value.integrity(),
                mode: value.mode(),
                redirect: value.redirect(),
                referrer: value.referrer(),
                referrer_policy: value.referrer_policy(),
            },
        }
    }
}
