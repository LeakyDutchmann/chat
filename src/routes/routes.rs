use crate::routes::chat::{websocket::handle_websocket, models::ChatMessage};
use crate::routes::route_models;
use super::history::fetch_history;
use super::auth::endpoints::{handle_registration, handle_authentication, handle_logout, get_me};
use crate::fileserver::serve_file;
use super::route_models::Route;
use crate::fileserver::get_path;

use sqlx::mysql::MySqlPool;
use tokio::sync::broadcast::Sender;
use tokio::net::TcpStream;
use crate::Shutdown;

pub async fn handle_routes(stream: TcpStream, buffer: &[u8], sender: Sender<ChatMessage>, db_pool: MySqlPool, shutdown: Sender<Shutdown>) {
    let route = route_models::Route::from_buffer(buffer);
    match route {
        Route::Init => {
            let result = serve_file(stream, &get_path("index.html"), "text/html").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Js => {
            let result = serve_file(stream, &get_path("script.js"), "application/javascript").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::CssReset => {
            let result = serve_file(stream, &get_path("reset.css"), "text/css").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::StyleCss => {
            let result = serve_file(stream, &get_path("style.css"), "text/css").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Icon => {
            let result = serve_file(stream, &get_path("icon.png"), "image/png").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Register(form) => {
            handle_registration(stream, db_pool, form).await;
        }
        Route::Login(form) => {
            handle_authentication(stream, db_pool, form).await;
        }
        Route::Logout => {
            handle_logout(stream, db_pool, buffer).await;
        }
        Route::History => {
            fetch_history(stream, db_pool).await;
        },
        Route::Me => {
            get_me(stream, db_pool, buffer).await;
        },
        Route::WebSocket => {
            handle_websocket(stream, buffer, sender, db_pool, shutdown).await;
        }
        Route::Unexpected(value) => {
            println!("Got unexpected route: {}", value);
        }
    };
}





