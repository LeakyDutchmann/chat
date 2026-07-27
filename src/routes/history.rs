use super::*;
use super::chat::models::ChatMessage;
use sqlx::mysql::MySqlPool;

pub async fn fetch_history(mut stream: TcpStream, db_pool: MySqlPool) {
    let rows_opt: Option<Vec<ChatMessage>> = sqlx::query_as("SELECT * FROM messages")
        .fetch_all(&db_pool)
        .await.ok();
    if let Some(rows) = rows_opt {
        let json = serde_json::to_string(&rows).unwrap();
        let response = format!("HTTP/1.1 OK 200\r\nContent-Type: application/json\r\n\r\n{}", json);
        println!("resp OK");
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }
}