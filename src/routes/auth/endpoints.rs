use super::*;
use crate::routes::auth::form::{AuthResponse, AuthForm};
use sqlx::mysql::MySqlPool;
use sqlx::Row;

use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2,
};
use rand_core::OsRng;

pub async fn handle_registration(mut stream: TcpStream, db_pool: MySqlPool, form: AuthForm) {
    let result = sqlx::query("SELECT username FROM users WHERE username = ?")
        .bind(&form.username)
        .fetch_one(&db_pool)
        .await.ok();
    if result.is_some() {
        let status = AuthResponse::from("error, user already exists", "Error, user already exists");
        let json = serde_json::to_string(&status).unwrap();
        let len = json.len();
        let response = format!("HTTP/1.1 OK 200\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", len, json);
        let _ = stream.write_all(response.as_bytes()).await;
        return;
    }
    let password_hash = hash_password(&form.password).expect("called unwrap on None password");
    let result = sqlx::query("INSERT INTO users(username, password_hash, color) values(?, ?, ?)")
        .bind(&form.username)
        .bind(password_hash)
        .bind(&form.color)
        .execute(&db_pool)
        .await;
    match result {
        Ok(_) => {
            println!("User saved to a db succesfully");
            let status = AuthResponse::from("ok", "User is succesfully registered");
            let json = serde_json::to_string(&status).unwrap();
            let len = json.len();
            let response = format!("HTTP/1.1 Ok 200\r\nContent-Type: application/json\r\nContent-length: {}\r\n\r\n{}", len, json);
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub fn hash_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).ok()?.to_string();
    Some(password_hash)
}

pub async fn handle_authentication(mut stream: TcpStream, db_pool: MySqlPool, form: AuthForm) {
    let row = sqlx::query("SELECT password_hash, color FROM users WHERE username =?")
        .bind(&form.username)
        .fetch_optional(&db_pool)
        .await.unwrap();
    if row.is_none() {
        let status = AuthResponse::from("error", "User does not exist");
        let status_str = serde_json::to_string(&status).unwrap();
        let len = status_str.len();
        let response = format!("HTTP/1.1 OK 200\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", len, status_str);
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    } else {
        let password_hash: String = row.unwrap().try_get("password_hash").unwrap();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        let argon2 = Argon2::default();
        if argon2.verify_password(form.password.as_bytes(), &parsed_hash).is_ok() {
            let status = AuthResponse::from("succesfully logged in", "Succesfully logged in");
            let status_str = serde_json::to_string(&status).unwrap();
            let len = status_str.len();
            let response = format!("HTTP/1.1 OK 200\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", len, status_str);

            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
        }
        
    }
}