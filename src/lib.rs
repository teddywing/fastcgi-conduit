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

trait From<T>: Sized {
    fn from(_: T) -> Self;
}

impl From<fastcgi::Request> for http::request::Builder {
    fn from(request: fastcgi::Request) -> Self {
        let method = request.param("REQUEST_METHOD")
            .unwrap_or("".to_owned());

        let uri = format!(
            "{}://{}{}",
            request.param("REQUEST_SCHEME").unwrap_or("".to_owned()),
            request.param("HTTP_HOST").unwrap_or("".to_owned()),
            request.param("REQUEST_URI").unwrap_or("".to_owned()),
        );

        return http::request::Builder::new()
            .method(&*method)
            .uri(&uri)

        // HTTP_* params become headers
    }
}
