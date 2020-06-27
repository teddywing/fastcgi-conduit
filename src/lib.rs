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
}

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;


struct FastCgiRequest<'a> {
    request: &'a fastcgi::Request,
    method: conduit::Method,
}

impl<'a> FastCgiRequest<'a> {
    pub fn new(request: &'a fastcgi::Request) -> RequestResult<Self> {
        let method = Self::method(request)
            .context(InvalidMethod)?;

        let r = Self {
            request: request,
            method: method,
        };

        r.parse();

        Ok(r)
    }

    fn parse(&self) {
        let headers = Self::headers_from_params(self.request.params());
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
