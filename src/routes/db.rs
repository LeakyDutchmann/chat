use sqlx::{self, mysql::{MySqlPool, MySqlPoolOptions}};
use std::time::Duration;
use crate::routes::chat::models::ChatMessage;

pub async fn estabilish_connection(db_url: &str) -> anyhow::Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(10))
        .connect(db_url)
        .await?;
    Ok(pool)
}

pub async fn save_to_db(message: ChatMessage, db: &MySqlPool) -> anyhow::Result<()> {
    let result = sqlx::query("INSERT INTO messages(room, username, message) VALUES(?, ?, ?)")
        .bind(message.room)
        .bind(message.username)
        .bind(message.message)
        .execute(db)
        .await?;
    if result.rows_affected() > 0 {
        println!("Message saved to database");
    }
    Ok(())
}
