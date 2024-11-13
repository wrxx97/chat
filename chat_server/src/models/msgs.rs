use chat_core::Message;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{AppError, AppState};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct CreateMessage {
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
}

impl AppState {
    pub async fn create_message(&self, input: &CreateMessage) -> Result<Message, AppError> {
        let msg = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, sender_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, chat_id, sender_id, content, created_at
            "#,
        )
        .bind(input.chat_id)
        .bind(input.sender_id)
        .bind(&input.content)
        .fetch_one(&self.pg_pool)
        .await?;

        Ok(msg)
    }

    pub async fn fetch_messages(&self, chat_id: i64) -> Result<Vec<Message>, AppError> {
        let msgs = sqlx::query_as(
            r#"
            SELECT * FROM messages
            WHERE chat_id = $1
            ORDER BY id DESC
            "#,
        )
        .bind(chat_id)
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(msgs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_message() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();

        let msg = CreateMessage {
            chat_id: 1,
            sender_id: 1,
            content: "test message".to_string(),
        };

        let ret = state.create_message(&msg).await.unwrap();

        assert_eq!(msg.chat_id, ret.chat_id);
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.content, "test message");
    }

    #[tokio::test]
    async fn test_fetch_messages() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();
        let msgs = state.fetch_messages(1).await.unwrap();

        assert_eq!(msgs.len(), 10);
    }
}
