use std::io;
use std::io::Write;

use conduit::Handler;

use snafu::{ResultExt, Snafu};

use crate::request;


const HTTP_VERSION: &'static str = "HTTP/1.1";


#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    WriteError { source: io::Error },

    #[snafu(display("Couldn't build request: {}", source))]
    RequestBuilder { source: request::Error },

    #[snafu(display("Couldn't parse response: {}", source))]
    ConduitResponse { source: conduit::BoxError },
}


pub struct Server;

impl Server {
    pub fn start<H: Handler + 'static + Sync>(handler: H) -> io::Result<Server> {
        fastcgi::run(move |mut raw_request| {
            match handle_request(&mut raw_request, &handler) {
                Ok(_) => (),

                // TODO: log
                // Ignore write errors as clients will have closed the
                // connection by this point.
                Err(Error::WriteError { .. }) => (),

                Err(Error::RequestBuilder { .. }) =>
                    internal_server_error(&mut raw_request.stdout()),
                Err(Error::ConduitResponse { .. }) =>
                    internal_server_error(&mut raw_request.stdout()),
            }
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
        "{} {} {}\r\n",
        HTTP_VERSION,
        head.status.as_str(),
        head.status.canonical_reason().unwrap_or("UNKNOWN"),
    )
        .context(WriteError)?;

    for (name, value) in head.headers.iter() {
        write!(&mut stdout, "{}: ", name).context(WriteError)?;
        stdout.write(value.as_bytes()).context(WriteError)?;
        stdout.write(b"\r\n").context(WriteError)?;
    }

    stdout.write(b"\r\n").context(WriteError)?;

    match body {
        conduit::Body::Static(slice) => stdout.write(slice).map(|_| ()).context(WriteError)?,
        conduit::Body::Owned(vec) => stdout.write(&vec).map(|_| ()).context(WriteError)?,
        conduit::Body::File(mut file) => io::copy(&mut file, &mut stdout).map(|_| ()).context(WriteError)?,
    };

    Ok(())
}

fn internal_server_error<W: Write>(mut w: W) {
    let code = conduit::StatusCode::INTERNAL_SERVER_ERROR;

    write!(
        w,
        "{} {} {}\r\n{}\r\n\r\n",
        HTTP_VERSION,
        code,
        code.canonical_reason().unwrap_or_default(),
        "Content-Length: 0",
    )
        .unwrap_or(())
}
