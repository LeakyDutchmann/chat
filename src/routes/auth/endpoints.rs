use super::*;
use form::{AuthForm, AuthResponse};
use crate::session_utils::{create_session, remove_session, verify_session, get_session_id};
use crate::http_utils::send_json;
use crate::log;

use sqlx::mysql::MySqlPool;
use sqlx::Row;
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2,
};
use rand_core::OsRng;
use tokio::net::TcpStream;

pub fn hash_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).ok()?.to_string();
    Some(password_hash)
}

pub async fn handle_registration(stream: TcpStream, db_pool: MySqlPool, form: AuthForm) {
    if form.username.is_empty() || form.password.is_empty() {
        send_json(
            AuthResponse::from_err("Empry field passed into form"),
            "400",
            "BAD REQUEST",
            None,
            stream
        ).await;
        return;
    }
    let result = sqlx::query("SELECT username FROM users WHERE username = ?")
        .bind(&form.username)
        .fetch_one(&db_pool)
        .await.ok();
    if result.is_some() {
        send_json(
            AuthResponse::from_err("Error, user already exists"),
            "400",
            "BAD REQUEST",
            None,
            stream,
        ).await;
        return;
    }
    let password_hash = hash_password(&form.password).expect("called unwrap on None password");
    let result = sqlx::query("INSERT INTO users(username, password_hash) values(?, ?)")
        .bind(&form.username)
        .bind(password_hash)
        .execute(&db_pool)
        .await;
    match result {
        Ok(_) => {
            let cookie = create_session(db_pool, form.username).await;
            let cookie_header = format!("Set-Cookie: {}", cookie);
            send_json(
                AuthResponse::from_ok("User is succesfully registered"),
                "200",
                "OK",
                Some(&cookie_header),
                stream,
            ).await;
        }
        Err(e) => {
            log(&format!("Error: {}", e), "red", false);
            send_json(
                AuthResponse::from_err("Internal server error"),
                "500",
                "INTERNAL SERVER ERROR",
                None,
                stream
            ).await;
        }
    }
}

pub async fn handle_authentication(stream: TcpStream, db_pool: MySqlPool, form: AuthForm) {
    if form.username.is_empty() || form.password.is_empty() {
        send_json(
            AuthResponse::from_err("Empry field passed into form"),
            "400",
            "BAD REQUEST",
            None,
            stream
        ).await;
        return;
    }
    let result = sqlx::query("SELECT password_hash FROM users WHERE username =?")
        .bind(&form.username)
        .fetch_optional(&db_pool)
        .await;
    if result.is_err() {
        send_json(
            AuthResponse::from_err("Internal server error"),
            "500",
            "INTERNAL SERVER ERROR",
            None,
            stream
        ).await;
        return;
    }
    let row_opt = result.unwrap();
    if let Some(row) = row_opt {
        let password_hash: String = row.try_get("password_hash").unwrap();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        let argon2 = Argon2::default();
        if argon2.verify_password(form.password.as_bytes(), &parsed_hash).is_ok() {         
            let cookie = create_session(db_pool, form.username).await;
            let cookie_header = format!("Set-Cookie: {}", cookie);
            send_json(
                AuthResponse::from_ok("Succesfully logged in"),
                "200",
                "OK",
                Some(&cookie_header),
                stream,
            ).await;
        } else {
            send_json(
                AuthResponse::from_err("Incorrect password"),
                "400",
                "BAD REQUEST",
                None,
                stream
            ).await;
        }
    } else {
        send_json(
            AuthResponse::from_err("User does not exist"),
            "400",
            "BAD REQUEST",
            None,
            stream
        ).await;
    }
}

pub async fn handle_logout(stream: TcpStream, db_pool: MySqlPool, buffer: &[u8]) {
    let session_id = get_session_id(buffer);
    if session_id.is_empty() {
        send_json(
            AuthResponse::from_err("Session ID is empty"),
            "400",
            "BAD REQUEST",
            None,
            stream,
        ).await;
        return;
    }
    let result = remove_session(session_id, db_pool).await;
    if result.is_err() {
        send_json(
            AuthResponse::from_err("Internal server error"),
            "500",
            "INTERNAL SERVER ERROR",
            None,
            stream,
        ).await;
    } else {
        send_json(
            AuthResponse::from_ok("Session is finished succesfully"),
            "200",
            "OK",
            None,
            stream
        ).await;
    }
}

pub async fn get_me(stream: TcpStream, db_pool: MySqlPool, buffer: &[u8]) {
    let session_id = get_session_id(buffer);
    if session_id.is_empty() {
        send_json(
            AuthResponse::from_err("Session ID is empty"),
            "400",
            "BAD REQUEST",
            None,
            stream,
        ).await;
        return;
    }
    if let Some(username) = verify_session(session_id, db_pool).await {
        send_json(
            AuthResponse::from_ok(username.trim()),
            "200",
            "OK",
            None,
            stream
        ).await;
    } else {
        send_json(
            AuthResponse::from_err("Session is not found"),
            "404",
            "NOT FOUND",
            None,
            stream,
        ).await;
    }
    
}