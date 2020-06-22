extern crate fastcgi;
extern crate http;

use std::io::Write;

use http::{Request, Response};
use http::request;


pub fn run<F, T>(handler: F)
where F: Fn(Request<()>) -> Response<T> + Send + Sync + 'static
{
    fastcgi::run(move |mut req| {
        // build request
        let r = request::Builder::new()
            .method("GET")
            .body(())
            .unwrap();

        handler(r);

        let params = req.params()
            .map(|(k, v)| k + ": " + &v)
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            &mut req.stdout(),
            "Content-Type: text/plain\n\n{}",
            params
        )
            .unwrap_or(());
    });
}
