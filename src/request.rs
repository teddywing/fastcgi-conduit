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
use std::io::Read;
use std::net::SocketAddr;

use inflector::cases::traincase::to_train_case;

use snafu::{ResultExt, Snafu};


/// Errors parsing a FastCGI request.
#[derive(Debug, Snafu)]
pub enum Error {
    /// The HTTP method is invalid.
    #[snafu(context(false))]
    InvalidMethod { source: http::method::InvalidMethod },

    /// An invalid HTTP header name.
    #[snafu(context(false))]
    InvalidHeaderName { source: conduit::header::InvalidHeaderName },

    /// An invalid HTTP header value.
    #[snafu(context(false))]
    InvalidHeaderValue { source: conduit::header::InvalidHeaderValue },

    /// An invalid remote address.
    #[snafu(context(false))]
    InvalidRemoteAddr { source: RemoteAddrError },
}

/// A convenience `Result` that contains a request `Error`.
pub type RequestResult<T, E = Error> = std::result::Result<T, E>;

/// Errors parsing an HTTP remote address.
#[derive(Debug, Snafu)]
pub enum RemoteAddrError {
    /// Error parsing the address part.
    #[snafu(display("Could not parse address {}: {}", address, source))]
    AddrParseError {
        address: String,
        source: std::net::AddrParseError,
    },

    /// Error parsing the port part.
    #[snafu(display("Could not parse port {}: {}", port, source))]
    PortParseError {
        port: String,
        source: std::num::ParseIntError
    },
}


/// Wraps a [`fastcgi::Request`][fastcgi::Request] to implement
/// [`conduit::RequestExt`][conduit::RequestExt].
///
/// [fastcgi::Request]: ../../fastcgi/struct.Request.html
/// [conduit::RequestExt]: ../../conduit/trait.RequestExt.html
pub struct FastCgiRequest<'a> {
    request: &'a mut fastcgi::Request,
    http_version: conduit::Version,
    host: String,
    method: conduit::Method,
    headers: conduit::HeaderMap,
    path: String,
    query: Option<String>,
    remote_addr: SocketAddr,
    content_length: Option<u64>,
    extensions: conduit::Extensions,
}

impl<'a> FastCgiRequest<'a> {
    /// Create a new `FastCgiRequest`.
    pub fn new(request: &'a mut fastcgi::Request) -> RequestResult<Self> {
        let version = Self::version(request);
        let host = Self::host(request);
        let method = Self::method(request)?;
        let headers = Self::headers(request.params())?;
        let path = Self::path(request);
        let query = Self::query(request);
        let remote_addr = Self::remote_addr(request)?;
        let content_length = Self::content_length(request);

        Ok(Self {
            request: request,
            http_version: version,
            host: host,
            method: method,
            headers: headers,
            path: path,
            query: query,
            remote_addr: remote_addr,
            content_length: content_length,
            extensions: conduit::TypeMap::new(),
        })
    }

    /// Extract the HTTP version.
    fn version(request: &fastcgi::Request) -> conduit::Version {
        match request.param("SERVER_PROTOCOL").unwrap_or_default().as_str() {
            "HTTP/0.9" => conduit::Version::HTTP_09,
            "HTTP/1.0" => conduit::Version::HTTP_10,
            "HTTP/1.1" => conduit::Version::HTTP_11,
            "HTTP/2.0" => conduit::Version::HTTP_2,
            "HTTP/3.0" => conduit::Version::HTTP_3,
            _ => conduit::Version::default(),
        }
    }

    /// Get the request scheme (HTTP or HTTPS).
    fn scheme(&self) -> conduit::Scheme {
        let scheme = self.request.param("REQUEST_SCHEME").unwrap_or_default();

        if scheme == "https" {
            conduit::Scheme::Https
        } else {
            conduit::Scheme::Http
        }
    }

    /// Get the HTTP host.
    ///
    /// This looks like `localhost:8000`.
    fn host(request: &fastcgi::Request) -> String {
        request.param("HTTP_HOST").unwrap_or_default()
    }

    /// Get the HTTP method (GET, HEAD, POST, etc.).
    fn method(
        request: &fastcgi::Request
    ) -> Result<conduit::Method, http::method::InvalidMethod> {
        conduit::Method::from_bytes(
            request.param("REQUEST_METHOD")
                .unwrap_or_default()
                .as_bytes()
        )
    }

    /// Build a map of request headers.
    fn headers(params: fastcgi::Params) -> RequestResult<conduit::HeaderMap> {
        let mut map = conduit::HeaderMap::new();
        let headers = Self::headers_from_params(params);

        for (name, value) in headers
            .iter()
            .map(|(name, value)| (name.as_bytes(), value.as_bytes()))
        {
            map.append(
                conduit::header::HeaderName::from_bytes(name)?,
                conduit::header::HeaderValue::from_bytes(value)?,
            );
        }

        Ok(map)
    }

    /// Extract headers from request params. Transform these into pairs of
    /// canonical header names and values.
    fn headers_from_params(params: fastcgi::Params) -> Vec<(String, String)> {
        return params
            .filter(|(key, _)| key.starts_with("HTTP_"))
            .map(|(key, value)| {
                let key = key.get(5..).unwrap_or_default();
                let key = &key.replace("_", "-");
                let key = &to_train_case(&key);

                (key.to_owned(), value)
            })
            .collect()
    }

    /// Get the URI path.
    ///
    /// Returns `/path` when the URI is `http://localhost:8000/path?s=query`.
    /// When the path is empty, returns `/`.
    fn path(request: &fastcgi::Request) -> String {
        match request.param("SCRIPT_NAME") {
            Some(p) => p,
            None => "/".to_owned(),
        }
    }

    /// Get the URI query string.
    ///
    /// Returns `s=query&lang=en` when the URI is
    /// `http://localhost:8000/path?s=query&lang=en`.
    fn query(request: &fastcgi::Request) -> Option<String> {
        request.param("QUERY_STRING")
    }

    /// Get the remote address of the request.
    fn remote_addr(request: &fastcgi::Request) -> Result<SocketAddr, RemoteAddrError> {
        let addr = request.param("REMOTE_ADDR").unwrap_or_default();
        let port = request.param("REMOTE_PORT").unwrap_or_default();

        Ok(
            SocketAddr::new(
                addr.parse().context(AddrParseError { address: addr })?,
                port.parse().context(PortParseError { port })?,
            )
        )
    }

    /// Get the request's content length.
    fn content_length(request: &fastcgi::Request) -> Option<u64> {
        request.param("CONTENT_LENGTH").and_then(|l| l.parse().ok())
    }
}

impl<'a> Read for FastCgiRequest<'a> {
    /// Read from the underlying FastCGI request's [`Stdin`][Stdin]
    ///
    /// [Stdin]: ../../fastcgi/struct.Stdin.html
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.request.stdin().read(buf)
    }
}

impl<'a> conduit::RequestExt for FastCgiRequest<'a> {
   fn http_version(&self) -> conduit::Version {
       self.http_version
   }

   fn method(&self) -> &conduit::Method {
       &self.method
   }

   fn scheme(&self) -> conduit::Scheme {
       self.scheme()
   }

   fn host(&self) -> conduit::Host<'_> {
       conduit::Host::Name(&self.host)
   }

   fn virtual_root(&self) -> std::option::Option<&str> {
       None
   }

   fn path(&self) -> &str {
       &self.path
   }

   fn query_string(&self) -> std::option::Option<&str> {
       self.query.as_ref()
           .map(|p| p.as_str())
   }

   fn remote_addr(&self) -> std::net::SocketAddr {
       self.remote_addr
   }

   fn content_length(&self) -> std::option::Option<u64> {
       self.content_length
   }

   fn headers(&self) -> &conduit::HeaderMap {
       &self.headers
   }

   fn body(&mut self) -> &mut (dyn std::io::Read) {
       self
   }

   fn extensions(&self) -> &conduit::Extensions {
       &self.extensions
   }

   fn mut_extensions(&mut self) -> &mut conduit::Extensions {
       &mut self.extensions
   }
}
