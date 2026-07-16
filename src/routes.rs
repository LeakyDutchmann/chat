use super::*;
use crate::fileserver::serve_file;

#[derive(Clone)]
pub enum Route {
    Init,
    Js,
    CssReset,
    StyleCss,
    Icon,
    Events,
    Unexpected(String)
}

struct RouteEntry {
    route: Route,
    path: &'static [u8]
}

static ROUTES: &[RouteEntry] = &[
    RouteEntry { path: b"GET / HTTP/1.1", route: Route::Init },
    RouteEntry { path: b"GET /script.js HTTP/1.1", route: Route::Js },
    RouteEntry { path: b"GET /reset.css HTTP/1.1", route: Route::CssReset },
    RouteEntry { path: b"GET /style.css HTTP/1.1", route: Route::StyleCss },
    RouteEntry { path: b"GET /favicon.ico HTTP/1.1", route: Route::Icon },
    RouteEntry { path: b"GET /events HTTP/1.1", route: Route::Events}
];

impl Route {
    pub fn from_buffer(buffer: &[u8]) -> Route {
        for route_ent in ROUTES {
            if buffer.starts_with(route_ent.path) {
                return route_ent.route.clone();
            }
        }
        Route::Unexpected(String::from_utf8_lossy(buffer).to_string())
    }
}

pub fn handle_route(stream: TcpStream, buffer: [u8; 1024]) {
    let route = Route::from_buffer(&buffer);
    match route {
        Route::Init => {
            let result = serve_file(stream, "static/index.html", "text/html");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Js => {
            let result = serve_file(stream, "static/script.js", "application/javascript");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::CssReset => {
            let result = serve_file(stream, "static/reset.css", "text/css");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::StyleCss => {
            let result = serve_file(stream, "static/style.css", "text/css");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Icon => {
            println!("got icon request");
        }
        Route::Events => {
            
        }
        Route::Unexpected(req) => {
            println!("got unexpected request: {}", req);
        }
    }
}