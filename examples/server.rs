extern crate http;

use std::fs::OpenOptions;
use std::io::prelude::*;

use http::{Response, StatusCode};

use fcgi;


fn main() {
    fcgi::run(move |req| {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("/tmp/fcgi-log.txt")
            .unwrap();
        write!(file, "» {:?}\n", req).unwrap();

        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(())
            .unwrap();

        return resp;
    });
}
