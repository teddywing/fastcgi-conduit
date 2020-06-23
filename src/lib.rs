extern crate fastcgi;
extern crate http;

use std::io::{BufReader, Write};

use http::{Request, Response};
use http::request;
use inflector::cases::traincase::to_train_case;


pub fn run<F, T>(handler: F)
where F: Fn(Request<()>) -> Response<T> + Send + Sync + 'static
{
    fastcgi::run(move |mut req| {
        let r: http::request::Builder = From::from(&req);

        handler(r.body(()).unwrap());

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

impl From<&fastcgi::Request> for http::request::Builder {
    fn from(request: &fastcgi::Request) -> Self {
        let method = request.param("REQUEST_METHOD")
            .unwrap_or("".to_owned());

        let uri = format!(
            "{}://{}{}",
            request.param("REQUEST_SCHEME").unwrap_or("".to_owned()),
            request.param("HTTP_HOST").unwrap_or("".to_owned()),
            request.param("REQUEST_URI").unwrap_or("".to_owned()),
        );

        let mut http_request = http::request::Builder::new()
            .method(&*method)
            .uri(&uri);

        let headers = headers_from_params(request.params());
        for (k, v) in headers {
            http_request = http_request.header(&k, &v);
        }

        // TODO: Add request body

        http_request

        // let body = BufReader::new(request.stdin());
        //
        // http_request.body(body)

        // HTTP_* params become headers
    }
}

fn headers_from_params(params: fastcgi::Params) -> Vec<(String, String)> {
    return params
        .filter(|(key, _)| key.starts_with("HTTP_"))
        .map(|(key, value)| {
            let mut key = key.get(5..).unwrap_or("").to_owned();
            key = key.replace("_", "-");
            key = to_train_case(&key);

            // Change _ to -
            // Uppercase each word

            (key, value)
        })
        .collect()
}
