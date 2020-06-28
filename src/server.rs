use std::io;
use std::io::Write;

use conduit::Handler;

use crate::request::FastCgiRequest;


pub struct Server;

impl Server {
    pub fn start<H: Handler + 'static + Sync>(handler: H) -> io::Result<Server> {
        fastcgi::run(move |mut raw_request| {
            handle_request(&mut raw_request, &handler);
        });

        Ok(Server{})
    }
}

fn handle_request<H>(
    mut raw_request: &mut fastcgi::Request,
    handler: &H,
) -> Result<(), ()>
where H: Handler + 'static + Sync
{
    let mut request = FastCgiRequest::new(&mut raw_request).unwrap();
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

    Ok(())
}
