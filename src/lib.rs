extern crate conduit;
extern crate fastcgi;
extern crate http;

mod request;

use std::io;
use std::io::Write;

use conduit::Handler;


pub struct Server;

impl Server {
    pub fn start<H: Handler + 'static + Sync>(handler: H) -> io::Result<Server> {
        fastcgi::run(move |mut raw_request| {
            let mut request = request::FastCgiRequest::new(&mut raw_request).unwrap();
            let response = handler.call(&mut request);

            let mut stdout = raw_request.stdout();

            let (head, body) = response.unwrap().into_parts();

            write!(
                &mut stdout,
                "HTTP/1.1 {} {}\r\n",
                head.status.as_str(),
                head.status.canonical_reason().unwrap_or("UNKNOWN"),
            );

            for (name, value) in head.headers.iter() {
                write!(&mut stdout, "{}: ", name).unwrap();
                stdout.write(value.as_bytes()).unwrap();
                stdout.write(b"\r\n").unwrap();
            }

            stdout.write(b"\r\n").unwrap();

            match body {
                conduit::Body::Static(slice) => stdout.write(slice).map(|_| ()).unwrap(),
                conduit::Body::Owned(vec) => stdout.write(&vec).map(|_| ()).unwrap(),
                conduit::Body::File(mut file) => io::copy(&mut file, &mut stdout).map(|_| ()).unwrap(),
            };
        });

        Ok(Server{})
    }
}
