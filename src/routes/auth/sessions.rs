use cookie::{Cookie, SameSite};
use sqlx::Row;
use sha1::{Sha1, Digest};
use rand::RngCore;
use super::*;

//returng cookie as a string, so I save space in caller function
pub async fn create_session(db_pool: MySqlPool, username: String) -> String {
    let session_id = create_session_id(&username).await;
    let _ = sqlx::query("INSERT INTO session(username, session_id) values(?, ?)")
        .bind(username)
        .bind(session_id.clone())
        .execute(&db_pool)
        .await.unwrap();
    println!("Session created: {}", session_id);
    let cookie = Cookie::build(("session", session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    cookie.to_string()
}

pub async fn create_session_id(username: &str) -> String {
    let mut bytes = [0u8; 32];
    let mut hasher = Sha1::new();
    rand::thread_rng().fill_bytes(&mut bytes);
    hasher.update(username.as_bytes());
    hasher.update(&bytes);

    let result = hasher.finalize();
    hex::encode(result)
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
                let (a, b) = value.split_once("=").unwrap();
                if a.trim() == "session" {
                    return b.trim().to_string();
                }
            }
        }
    }
    String::from("")
}


pub async fn verify_session(session_id: String, db_pool: MySqlPool) -> Option<(String, String)> {
    println!("Verifying session: {}", session_id);
    let row = sqlx::query("SELECT username, color FROM users WHERE username = (SELECT username FROM session WHERE session_id = ?)")
        .bind(session_id)
        .fetch_optional(&db_pool)
        .await.unwrap().unwrap();
    let username: String = row.try_get("username").ok()?;
    let color: String = row.try_get("color").ok()?;
    println!("Session verified: {}", username);
    Some((username, color))
}

pub async fn remove_session(session_id: String, db_pool: MySqlPool) {
    let result = sqlx::query("DELETE FROM session WHERE session_id =?")
        .bind(&session_id)
        .execute(&db_pool)
        .await.unwrap();
    if result.rows_affected() > 0 {
        println!("Session removed: {}", &session_id);
    } else {
        println!("Session not found: {}", &session_id);
    }
}

pub async fn cleanup_sessions(db_pool: &MySqlPool) {
    let result = sqlx::query("DELETE FROM session")
        .execute(db_pool)
        .await.unwrap();
    if result.rows_affected() > 0 {
        println!("cleaned up {} session(s)", result.rows_affected());
    }
}