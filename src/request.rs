use std::io;
use std::io::Read;
use std::net::SocketAddr;

use inflector::cases::traincase::to_train_case;

use snafu::{ResultExt, Snafu};


#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    InvalidMethod { source: http::method::InvalidMethod },

    #[snafu(display("{}", source))]
    InvalidHeaderName { source: conduit::header::InvalidHeaderName },

    #[snafu(display("{}", source))]
    InvalidHeaderValue { source: conduit::header::InvalidHeaderValue },

    #[snafu(display("{}", source))]
    InvalidRemoteAddr { source: RemoteAddrError },
}

pub type RequestResult<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum RemoteAddrError {
    #[snafu(display("Could not parse address {}: {}", address, source))]
    AddrParseError {
        address: String,
        source: std::net::AddrParseError,
    },

    #[snafu(display("Could not parse port {}: {}", port, source))]
    PortParseError {
        port: String,
        source: std::num::ParseIntError
    },
}


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
    pub fn new(request: &'a mut fastcgi::Request) -> RequestResult<Self> {
        let version = Self::version(request);
        let host = Self::host(request);
        let method = Self::method(request).context(InvalidMethod)?;
        let headers = Self::headers(request.params())?;
        let path = Self::path(request);
        let query = Self::query(request);
        let remote_addr = Self::remote_addr(request).context(InvalidRemoteAddr)?;
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

    fn scheme(&self) -> conduit::Scheme {
        let scheme = self.request.param("REQUEST_SCHEME").unwrap_or_default();

        if scheme == "https" {
            conduit::Scheme::Https
        } else {
            conduit::Scheme::Http
        }
    }

    fn host(request: &fastcgi::Request) -> String {
        request.param("HTTP_HOST").unwrap_or_default()
    }

    fn method(
        request: &fastcgi::Request
    ) -> Result<conduit::Method, http::method::InvalidMethod> {
        conduit::Method::from_bytes(
            request.param("REQUEST_METHOD")
                .unwrap_or_default()
                .as_bytes()
        )
    }

    fn headers(params: fastcgi::Params) -> RequestResult<conduit::HeaderMap> {
        let mut map = conduit::HeaderMap::new();
        let headers = Self::headers_from_params(params);

        for (name, value) in headers
            .iter()
            .map(|(name, value)| (name.as_bytes(), value.as_bytes()))
        {
            map.append(
                conduit::header::HeaderName::from_bytes(name)
                    .context(InvalidHeaderName)?,
                conduit::header::HeaderValue::from_bytes(value)
                    .context(InvalidHeaderValue)?,
            );
        }

        Ok(map)
    }

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

    fn path(request: &fastcgi::Request) -> String {
        match request.param("SCRIPT_NAME") {
            Some(p) => p,
            None => "/".to_owned(),
        }
    }

    fn query(request: &fastcgi::Request) -> Option<String> {
        request.param("QUERY_STRING")
    }

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

    fn content_length(request: &fastcgi::Request) -> Option<u64> {
        request.param("CONTENT_LENGTH").and_then(|l| l.parse().ok())
    }
}

impl<'a> Read for FastCgiRequest<'a> {
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
