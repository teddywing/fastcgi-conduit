extern crate http;

use http::{Response, StatusCode};

use fcgi_rs;


fn main() {
    fcgi_rs::run(|req| {
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(())
            .unwrap();

        return resp;
    });
}
