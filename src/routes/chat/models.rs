use super::*;
use sqlx::{mysql::MySqlRow, FromRow, Row};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub room: String,
    pub username: String,
    pub message: String
}

#[derive(Deserialize)]
pub struct RawChatMessage {
    pub room: String,
    pub message: String,
}

impl ChatMessage {
    pub fn from_form(form: String) -> Option<ChatMessage> {
        let parts: Vec<String> = form.split("&").map(|p| p.to_string()).collect();
        let mut parsed = ChatMessage {
            room: String::new(),
            username: String::new(),
            message: String::new(),
        };
        for part in parts {
            if let Some((a, b)) = part.split_once("=") {
                let mut decoded = decode(b.trim()).ok()?.to_string();
                decoded = decoded.replace("+", " ");
                if decoded.contains('+') {
                    println!("invalid character: {}", decoded);
                }
                match a {
                    "room" => parsed.room = decoded,
                    "username" => parsed.username = decoded,
                    "message" => parsed.message = decoded,
                    _ => { continue}
                }
            }
        }
        Some(parsed)
    }
    pub fn from_raw(raw: RawChatMessage, username: &str) -> ChatMessage {
        ChatMessage {
            room: raw.room,
            username: username.to_string(),
            message: raw.message,
        }
    }
}

impl FromRow<'_, MySqlRow> for ChatMessage {
    fn from_row(row: &MySqlRow) -> Result<ChatMessage, sqlx::Error> {
        let room: String = row.try_get("room")?;
        let username: String = row.try_get("username")?;
        let message: String = row.try_get("message")?;
        Ok(ChatMessage {
            room: room,
            username: username,
            message: message,
        })
    }
}