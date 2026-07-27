use crate::routes::http::{parse_request_body};
use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub message: String,
}

impl AuthResponse {
    pub fn from(status: &str, message: &str) -> AuthResponse {
        AuthResponse {
            status: status.to_string(),
            message: message.to_string(),
        }
    }
}


#[derive(Clone, Debug)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
    pub color: String
}

impl AuthForm {
    pub fn from_buffer(buffer: &[u8]) -> AuthForm {
        let body = parse_request_body(String::from_utf8_lossy(buffer).to_string());
        let mut form = AuthForm {
            username: String::new(),
            password: String::new(),
            color: String::new()
        };
        let parts: Vec<&str> = body.split('&').collect();
        for part in parts {
            let (a, b) = part.split_once("=").unwrap();
            let decoded = decode(b.trim()).unwrap().replace("+", " ");
            match a {
                "username" => {
                    form.username = decoded;
                }
                "password" => {
                    form.password = decoded;
                }
                "color" => {
                    form.color = decoded;
                }
                _ => {
                    continue;
                }
            }
        }
        form
    }
}