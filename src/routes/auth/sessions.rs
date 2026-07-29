use cookie::{Cookie, SameSite};
use super::*;

//returng cookie as a string, so I save space in caller function
pub async fn create_session(db_pool: MySqlPool, username: String) -> String {
    let session_id = create_session_id(&username).await;
    let _ = sqlx::query("INSERT INTO session(username, session_id) values(?, ?)")
        .bind(username)
        .bind(session_id.clone())
        .execute(&db_pool)
        .await.unwrap();
    let cookie = Cookie::build(("session", session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    cookie.to_string()
}

pub async fn create_session_id(username: &str) -> String {
    let now = chrono::Utc::now();
    let time =  now.time().to_string();
    let raw = time + username + "cookie";
    raw
}

pub fn get_session_id(buffer: &[u8]) -> String {
    let buffer_str = String::from_utf8_lossy(&buffer).to_string();
    let lines: Vec<String> = buffer_str.split("\n").map(|s| s.to_string()).collect();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(":") {
            if key == "Cookie" {
                println!("got sweet cookie: {}", value);
            }
        }
    }
    String::from("")
}

