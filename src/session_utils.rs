use super::log;

use cookie::{Cookie, SameSite};
use sha1::{Sha1, Digest};
use rand::RngCore;
use sqlx::{self, mysql::MySqlPool, Row};

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
    let mut bytes = [0u8; 32];
    let mut hasher = Sha1::new();
    rand::thread_rng().fill_bytes(&mut bytes);
    hasher.update(username.as_bytes());
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn get_session_id(buffer: &[u8]) -> String {
    let buffer_str = String::from_utf8_lossy(buffer).to_string();
    let lines: Vec<String> = buffer_str.split("\n").map(|s| s.to_string()).collect();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(":") && key == "Cookie" {
            let (a, b) = value.split_once("=").unwrap();
            if a.trim() == "session" {
                return b.trim().to_string();
            }
        }
    }
    String::from("")
}

pub async fn verify_session(session_id: String, db_pool: MySqlPool) -> Option<String> {
    let row = sqlx::query("SELECT username FROM users WHERE username = (SELECT username FROM session WHERE session_id = ?)")
        .bind(session_id)
        .fetch_optional(&db_pool)
        .await.unwrap();
    row.as_ref()?;
    let row = row.unwrap();
    let username: String = row.try_get("username").ok()?;
    Some(username)
}

pub async fn remove_session(session_id: String, db_pool: MySqlPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM session WHERE session_id =?")
        .bind(&session_id)
        .execute(&db_pool)
        .await.unwrap();
    if result.rows_affected() > 0 {
        Ok(result.rows_affected())
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn cleanup_sessions(db_pool: &MySqlPool) {
    let result = sqlx::query("DELETE FROM session")
        .execute(db_pool)
        .await.unwrap();
    if result.rows_affected() > 0 {
        log(format!("cleaned up {} session(s)", result.rows_affected()).as_str(), "blue", false);
    }
}