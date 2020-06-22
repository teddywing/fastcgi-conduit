extern crate http;

use http::{Response, StatusCode};

use fcgi;


fn main() {
    fcgi::run(|req| {
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(())
            .unwrap();

        return resp;
    });
}
