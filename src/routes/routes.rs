use crate::routes::chat::{websocket::handle_websocket, models::ChatMessage};
use super::*;
use fileserver::serve_file;
use route_models::Route;
use sqlx::mysql::MySqlPool;
use tokio::sync::broadcast::Sender;
use history::fetch_history;
use auth::endpoints::{handle_registration, handle_authentication};


pub async fn handle_routes(stream: TcpStream, buffer: &[u8], sender: Sender<ChatMessage>, db_pool: MySqlPool) {
    let route = routes::Route::from_buffer(&buffer);
    match route {
        Route::Init => {
            let result = serve_file(stream, "static/index.html", "text/html").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Js => {
            let result = serve_file(stream, "static/script.js", "application/javascript").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::CssReset => {
            let result = serve_file(stream, "static/reset.css", "text/css").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::StyleCss => {
            let result = serve_file(stream, "static/style.css", "text/css").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Icon => {
            return;
        }
        Route::Register(form) => {
            handle_registration(stream, db_pool, form).await;
        }
        Route::Login(form) => {
            handle_authentication(stream, db_pool, form).await;
        }
        Route::History => {
            fetch_history(stream, db_pool).await;
        },
        Route::WebSocket => {
            handle_websocket(stream, buffer, sender, db_pool).await;
        }
        Route::Unexpected(value) => {
            println!("Got unexpected route: {}", value);
        }
    };
}





