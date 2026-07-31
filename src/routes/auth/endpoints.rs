use super::*;
use crate::routes::auth::{form::{AuthForm, AuthResponse}, sessions::get_session_id};
use sessions::{create_session, remove_session, verify_session};
use sqlx::Row;
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2,
};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).ok()?.to_string();
    Some(password_hash)
}

pub async fn handle_registration(mut stream: TcpStream, db_pool: MySqlPool, form: AuthForm) {
    if form.username.is_empty() || form.password.is_empty() {
        let status = AuthResponse::from("error", "Empty field passed");
        let status_str = serde_json::to_string(&status).unwrap();
        let len = status_str.len();
        let pesponse = format!("HTTP/1.1 400 Bad request\r\nContent-Type: application/json\r\nContent-length: {}\r\n\r\n{}", len, status_str);
        let _ = stream.write_all(pesponse.as_bytes()).await;
        let _ = stream.flush().await;
        return;
    }
    let result = sqlx::query("SELECT username FROM users WHERE username = ?")
        .bind(&form.username)
        .fetch_one(&db_pool)
        .await.ok();
    if result.is_some() {
        let status = AuthResponse::from("error, user already exists", "Error, user already exists");
        let json = serde_json::to_string(&status).unwrap();
        let len = json.len();
        let response = format!("HTTP/1.1 400 Bad request\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", len, json);
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
            let status = AuthResponse::from("ok", "User is succesfully registered");
            let json = serde_json::to_string(&status).unwrap();
            let len = json.len();
            let cookie = create_session(db_pool, form.username).await;
            let response = format!("HTTP/1.1 200 ok\r\nSet-Cookie: {}\r\nContent-Type: application/json\r\nContent-length: {}\r\n\r\n{}", cookie, len, json);
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
        }
        Err(e) => {
            println!("Error: {}", e);
            let status = AuthResponse::from("error", "Internal server error");
            let json = serde_json::to_string(&status).unwrap();
            let len = json.len();
            let response = format!("HTTP/1.1 400 Bad request\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", len, json);
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
        }
    }
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
            let cookie = create_session(db_pool, form.username).await;
            let response = format!("HTTP/1.1 OK 200\r\nContent-Type: application/json\r\nContent-Length: {}\r\nSet-Cookie: {}\r\n\r\n{}", len, cookie, status_str);
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
        }
        
    }
}

pub async fn handle_logout(mut stream: TcpStream, db_pool: MySqlPool, buffer: &[u8]) {
    let session_id = get_session_id(buffer);
    remove_session(session_id, db_pool).await;
    let status = AuthResponse::from("Ok", "Session deleted succesfully");
    let status_str = serde_json::to_string(&status).unwrap();
    let len = status_str.len();
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", len, status_str);
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}

pub async fn get_me(mut stream: TcpStream, db_pool: MySqlPool, buffer: &[u8]) {
    let session_id = get_session_id(buffer);
    if let Some((username, color)) = verify_session(session_id, db_pool).await {
        let line = username + ":" + &color;
        let status = AuthResponse::from("ok", &line.trim());
        let status_str = serde_json::to_string(&status).unwrap();
        let len = status_str.len();
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",len, status_str);
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await; 
    } else {
        let status = AuthResponse::from("error", "Session not found");
        let status_str = serde_json::to_string(&status).unwrap();
        let len = status_str.len();
        let response = format!("HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",len, status_str);
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }
    
}