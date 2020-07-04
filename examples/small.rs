use conduit::{header, Body, RequestExt, Response};
use fastcgi_conduit::Server;


fn main() {
    Server::start(handler).unwrap();
}

fn handler(_req: &mut dyn RequestExt) -> std::io::Result<Response<Body>> {
    Ok(
        Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from_static(b"<h1>Hello</h1>"))
            .unwrap()
    )
}
