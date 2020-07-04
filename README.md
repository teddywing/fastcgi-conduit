fastcgi-conduit
===============

[Documentation][fastcgi-conduit-docs]

A [Conduit][conduit] interface for the [fastcgi] crate. This allows an
application to run over FastCGI with an API that’s easier to use.


## Example

``` rust
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
```


## License
Copyright © 2020 Teddy Wing. Licensed under the GNU GPLv3+ (see the included
COPYING file).


[fastcgi-conduit-docs]: https://teddywing.github.io/fastcgi-conduit/fastcgi_conduit/
[conduit]: https://lib.rs/crates/conduit
[fastcgi]: https://lib.rs/crates/fastcgi
