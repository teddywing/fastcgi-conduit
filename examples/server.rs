extern crate conduit;
extern crate conduit_router;
extern crate http;
extern crate fastcgi_conduit;

use std::io;

use conduit::{Body, HttpResult, RequestExt, Response};
use conduit::header;
use conduit_router::{RequestParams, RouteBuilder};

use fastcgi_conduit::Server;


fn main() {
    let mut router = RouteBuilder::new();

    router.get("/", handler);
    router.get("/:var", var_handler);

    Server::start(router);
}

fn handler(_req: &mut dyn RequestExt) -> io::Result<Response<Body>> {
    Ok(
        Response::builder()
            .status(202)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from_static(b"<h1>Test</h1>"))
            .unwrap()
    )
}

fn var_handler(req: &mut dyn RequestExt) -> HttpResult {
    let var = req.params().find("var").unwrap();
    let text = format!("The value is: {}", var).into_bytes();

    Response::builder().body(Body::from_vec(text))
}
