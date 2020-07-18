// Copyright (c) 2020  Teddy Wing
//
// This file is part of FastCGI-Conduit.
//
// FastCGI-Conduit is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// FastCGI-Conduit is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with FastCGI-Conduit. If not, see <https://www.gnu.org/licenses/>.

use std::io;
use std::io::Write;

use conduit::Handler;

use log::error;

use snafu::{ResultExt, Snafu};

use crate::request;


/// Wraps server errors.
#[derive(Debug, Snafu)]
pub enum Error {
    /// I/O write errors during response output.
    #[snafu(context(false))]
    Write { source: io::Error },

    /// Error building the request into a [`FastCgiRequest`][FastCgiRequest].
    ///
    /// [FastCgiRequest]: ../request/struct.FastCgiRequest.html
    #[snafu(display("Couldn't build request: {}", source))]
    RequestBuilder { source: request::Error },

    /// Error building a [`conduit::Response`][conduit::Response].
    ///
    /// [conduit::Response]: ../../conduit/struct.Response.html
    #[snafu(display("Couldn't parse response: {}", source))]
    ConduitResponse { source: conduit::BoxError },
}


/// The application server that interfaces with FastCGI.
pub struct Server;

impl Server {
    /// Start the server.
    ///
    /// Start the main [`fastcgi::run`][fastcgi::run] process to listen for
    /// requests and handle them using `handler`.
    ///
    /// [fastcgi::run]: ../../fastcgi/fn.run.html
    pub fn start<H: Handler + 'static + Sync>(handler: H) -> Server {
        fastcgi::run(move |mut raw_request| {
            match handle_request(&mut raw_request, &handler) {
                Ok(_) => (),
                Err(e) => match e {
                    // Ignore write errors as clients will have closed the
                    // connection by this point.
                    Error::Write { .. } => error!("Write error: {}", e),

                    Error::RequestBuilder { .. } => {
                        error!("Unable to build request: {}", e);

                        internal_server_error(&mut raw_request.stdout())
                    },
                    Error::ConduitResponse { .. } => {
                        error!("Error getting response: {}", e);

                        internal_server_error(&mut raw_request.stdout())
                    },
                }
            }
        });

        Server{}
    }
}

/// Given a raw FastCGI request and a Conduit handler, get a response from the
/// handler to write to a FastCGI response.
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
        "Status: {}\r\n",
        head.status.as_str(),
    )?;

    for (name, value) in head.headers.iter() {
        write!(&mut stdout, "{}: ", name)?;
        stdout.write(value.as_bytes())?;
        stdout.write(b"\r\n")?;
    }

    stdout.write(b"\r\n")?;

    match body {
        conduit::Body::Static(slice) =>
            stdout.write(slice).map(|_| ())?,
        conduit::Body::Owned(vec) =>
            stdout.write(&vec).map(|_| ())?,
        conduit::Body::File(mut file) =>
            io::copy(&mut file, &mut stdout).map(|_| ())?,
    };

    Ok(())
}

/// Write a 500 internal server error to `w`.
fn internal_server_error<W: Write>(mut w: W) {
    let code = conduit::StatusCode::INTERNAL_SERVER_ERROR;

    write!(
        w,
        "Status: {}\r\n{}\r\n\r\n",
        code,
        "Content-Length: 0",
    )
        .unwrap_or_else(|e| error!("Write error: {}", e))
}
