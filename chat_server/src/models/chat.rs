use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::AppError;

use super::{Chat, ChatType, ChatUser};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, Default)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, Default)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[allow(unused)]
impl Chat {
    fn get_name_from_members(members: Vec<ChatUser>) -> String {
        members
            .into_iter()
            .map(|u| u.fullname)
            .collect::<Vec<_>>()
            .join(",")
    }
    pub async fn create(input: CreateChat, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let users = ChatUser::fetch_by_ids(&input.members, pool).await?;
        let len = users.len();

        let chat_type = match (&input.name, len) {
            (_, 0..=1) => {
                return Err(AppError::CreateChatError(
                    "At least 2 members are required".to_string(),
                ))
            }
            (None, 9..) => {
                return Err(AppError::CreateChatError(
                    "Name is required when members are more than 8".to_string(),
                ))
            }
            (None, 2) => ChatType::Single,
            (None, 3..=8) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat: Self = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id)
        .bind(input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_all(ws_id: i64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT * FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    pub async fn update_by_id(id: i64, input: UpdateChat, pool: &PgPool) -> Result<Self, AppError> {
        let chat: Self = sqlx::query_as(
            r#"
            UPDATE chats
            SET name = $1, members = $2, public = $3
            WHERE id = $3
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(input.name)
        .bind(input.members)
        .bind(input.public)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete_by_id(id: i64, pool: &PgPool) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_pool;

    #[tokio::test]
    async fn create_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = CreateChat::new("", &[1, 2], false);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("chat create failed");
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
    }

    #[tokio::test]
    async fn fetch_all_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all(1, &pool)
            .await
            .expect("fetch all chats failed");

        assert_eq!(chats.len(), 4);
    }
}
