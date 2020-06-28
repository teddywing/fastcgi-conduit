extern crate conduit;
extern crate http;
extern crate fastcgi_conduit;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;

use conduit::{Body, RequestExt, Response};
use conduit::header;

use fastcgi_conduit::Server;


fn main() {
    Server::start(handler);
}

fn handler(req: &mut dyn RequestExt) -> io::Result<Response<Body>> {
    Ok(
        Response::builder()
            .status(202)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from_static(b"<h1>Test</h1>"))
            .unwrap()
    )
}
