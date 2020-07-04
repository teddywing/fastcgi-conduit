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

extern crate conduit;
extern crate conduit_router;
extern crate http;
extern crate fastcgi_conduit;

use std::io;

use conduit::{Body, HttpResult, RequestExt, Response};
use conduit::header;
use conduit_router::{RequestParams, RouteBuilder};

use fastcgi_conduit::Server;


fn main() {
    let mut router = RouteBuilder::new();

    router.get("/", handler);
    router.get("/:var", var_handler);

    Server::start(router);
}

fn handler(_req: &mut dyn RequestExt) -> io::Result<Response<Body>> {
    Ok(
        Response::builder()
            .status(202)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from_static(b"<h1>Test</h1>"))
            .unwrap()
    )
}

fn var_handler(req: &mut dyn RequestExt) -> HttpResult {
    let var = req.params().find("var").unwrap();
    let text = format!("The value is: {}", var).into_bytes();

    Response::builder().body(Body::from_vec(text))
}
