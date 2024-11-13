mod chat;
mod file;
mod msgs;
mod user;
mod workspace;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use chat::{CreateChat, UpdateChat};
pub use msgs::CreateMessage;
pub use user::{CreateUser, SignInUser};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct ChatFile {
    pub ws_id: i64,
    pub ext: String, // extract ext from filename
    pub hash: String,
}
