//! Wrapper around the `fetch` API.
//!
//! # Example
//!
//! ```
//! # use gloo_net::http::Request;
//! # async fn no_run() {
//! let resp = Request::get("/path")
//!     .send()
//!     .await
//!     .unwrap();
//! assert_eq!(resp.status(), 200);
//! # }
//! ```

#[macro_use]
mod request;
mod body;
mod headers;
mod response;

#[doc(inline)]
pub use http::Method;

pub use request::{Request, RequestBuilder};
pub use response::{Response, ResponseBuilder};
