use std::io;
use std::io::Write;

use conduit::Handler;

use snafu::{ResultExt, Snafu};

use crate::request;


#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    Io { source: io::Error },

    #[snafu(display("Couldn't build request: {}", source))]
    RequestBuilder { source: request::Error },

    #[snafu(display("Couldn't parse response: {}", source))]
    ConduitResponse { source: conduit::BoxError },
}


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
) -> Result<(), Error>
where H: Handler + 'static + Sync
{
    let mut request = request::FastCgiRequest::new(&mut raw_request)
        .context(RequestBuilder)?;
    let response = handler.call(&mut request);

    let mut stdout = raw_request.stdout();

    let (head, body) = response
        .context(ConduitResponse)?
        .into_parts();

    write!(
        &mut stdout,
        "HTTP/1.1 {} {}\r\n",
        head.status.as_str(),
        head.status.canonical_reason().unwrap_or("UNKNOWN"),
    )
        .context(Io)?;

    for (name, value) in head.headers.iter() {
        write!(&mut stdout, "{}: ", name).context(Io)?;
        stdout.write(value.as_bytes()).context(Io)?;
        stdout.write(b"\r\n").context(Io)?;
    }

    stdout.write(b"\r\n").context(Io)?;

    match body {
        conduit::Body::Static(slice) => stdout.write(slice).map(|_| ()).context(Io)?,
        conduit::Body::Owned(vec) => stdout.write(&vec).map(|_| ()).context(Io)?,
        conduit::Body::File(mut file) => io::copy(&mut file, &mut stdout).map(|_| ()).context(Io)?,
    };

    Ok(())
}
