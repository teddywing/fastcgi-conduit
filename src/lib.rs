#![warn(missing_docs)]

//! # fastcgi-conduit
//!
//! FastCGI-Conduit provides a [Conduit] interface to FastCGI, enabling a
//! high-level API for FastCGI applications.
//!
//!
//! ## Example
//!
//! ``` rust
//! use conduit::{header, Body, RequestExt, Response};
//! use fastcgi_conduit::Server;
//!
//!
//! fn main() {
//!     Server::start(handler).unwrap();
//! }
//!
//! fn handler(_req: &mut dyn RequestExt) -> std::io::Result<Response<Body>> {
//!     Ok(
//!         Response::builder()
//!             .header(header::CONTENT_TYPE, "text/html")
//!             .body(Body::from_static(b"<h1>Hello</h1>"))
//!             .unwrap()
//!     )
//! }
//! ```
//!
//!
//! [Conduit]: ../conduit/index.html

extern crate conduit;
extern crate fastcgi;
extern crate http;
extern crate log;

mod request;
mod server;

pub use server::Server;
