use super::auth::form::AuthForm;

#[derive(Clone, Debug)]
pub enum Route {
    Init,
    Js,
    CssReset,
    StyleCss,
    Icon,
    WebSocket,
    History,
    Register(AuthForm),
    Login(AuthForm),
    Logout,
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
    RouteEntry { path: b"GET /history HTTP/1.1", route: Route::History },
    RouteEntry { path: b"GET /ws HTTP/1.1", route: Route::WebSocket},
    RouteEntry { path: b"POST /logout HTTP/1.1", route: Route::Logout},
];

impl Route {
    pub fn from_buffer(buffer: &[u8]) -> Route {
        if buffer.starts_with(b"POST /register HTTP/1.1") {
            return Route::Register(AuthForm::from_buffer(buffer));
        }
        if buffer.starts_with(b"POST /login HTTP/1.1") {
            return Route::Login(AuthForm::from_buffer(buffer));
        }
        for route_ent in ROUTES {
            if buffer.starts_with(route_ent.path) {
                return route_ent.route.clone();
            }
        }
        Route::Unexpected(String::from_utf8_lossy(buffer).to_string())
    }
}