mod chat;
mod file;
mod user;
mod workspace;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use chat::{CreateChat, UpdateChat};
pub use user::{CreateUser, SignInUser};

#[derive(Serialize, Deserialize, Debug, FromRow, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow, Clone, PartialEq)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, FromRow, Clone, PartialEq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct ChatFile {
    pub ws_id: i64,
    pub ext: String, // extract ext from filename
    pub hash: String,
}

#[cfg(test)]

impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
            password_hash: None,
        }
    }
}
