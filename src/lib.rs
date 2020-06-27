extern crate conduit;
extern crate fastcgi;
extern crate http;

use std::io;
use std::io::{BufReader, Write};

use conduit::Handler;

use inflector::cases::traincase::to_train_case;

use snafu::{ResultExt, Snafu};


#[derive(Debug, Snafu)]
pub enum RequestError {
    #[snafu(display("{}", source))]
    InvalidMethod { source: http::method::InvalidMethod },

    #[snafu(display("{}", source))]
    InvalidHeaderName { source: conduit::header::InvalidHeaderName },

    #[snafu(display("{}", source))]
    InvalidHeaderValue { source: conduit::header::InvalidHeaderValue },
}

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;


struct FastCgiRequest<'a> {
    request: &'a fastcgi::Request,
    http_version: conduit::Version,
    method: conduit::Method,
    headers: conduit::HeaderMap,
}

impl<'a> FastCgiRequest<'a> {
    pub fn new(request: &'a fastcgi::Request) -> RequestResult<Self> {
        let method = Self::method(request)
            .context(InvalidMethod)?;

        let headers = Self::headers(request.params())?;

        let r = Self {
            request: request,
            http_version: Self::version(&request),
            method: method,
            headers: headers,
        };

        Ok(r)
    }

    fn version(request: &'a fastcgi::Request) -> conduit::Version {
        match request.param("SERVER_PROTOCOL").unwrap_or_default().as_str() {
            "HTTP/0.9" => conduit::Version::HTTP_09,
            "HTTP/1.0" => conduit::Version::HTTP_10,
            "HTTP/1.1" => conduit::Version::HTTP_11,
            "HTTP/2.0" => conduit::Version::HTTP_2,
            "HTTP/3.0" => conduit::Version::HTTP_3,
            _ => conduit::Version::default(),
        }
    }

    fn method(
        request: &'a fastcgi::Request
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
}

// impl<'a> conduit::RequestExt for FastCgiRequest {
//    fn http_version(&self) -> conduit::Version { todo!() }
//    fn method(&self) -> &conduit::Method {
//        self.method
//    }
//    fn scheme(&self) -> conduit::Scheme { todo!() }
//    fn host(&'a self) -> conduit::Host<'a> { todo!() }
//    fn virtual_root(&'a self) -> std::option::Option<&'a str> { todo!() }
//    fn path(&'a self) -> &'a str { todo!() }
//    fn query_string(&'a self) -> std::option::Option<&'a str> { todo!() }
//    fn remote_addr(&self) -> std::net::SocketAddr { todo!() }
//    fn content_length(&self) -> std::option::Option<u64> { todo!() }
//    fn headers(&self) -> &conduit::HeaderMap { todo!() }
//    fn body(&'a mut self) -> &'a mut (dyn std::io::Read + 'a) { todo!() }
//    fn extensions(&'a self) -> &'a conduit::TypeMap { todo!() }
//    fn mut_extensions(&'a mut self) -> &'a mut conduit::TypeMap { todo!() }
// }


struct Server;

impl Server {
    pub fn start<H: Handler + 'static + Sync>(handler: H) -> io::Result<Server> {
        Ok(Server{})
    }
}
