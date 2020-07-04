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

use conduit::{header, Body, RequestExt, Response};
use fastcgi_conduit::Server;


fn main() {
    Server::start(handler);
}

fn handler(_req: &mut dyn RequestExt) -> std::io::Result<Response<Body>> {
    Ok(
        Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from_static(b"<h1>Hello</h1>"))
            .unwrap()
    )
}
