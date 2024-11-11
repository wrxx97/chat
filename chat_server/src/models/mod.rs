mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use user::{CreateUser, SignInUser};

#[derive(Serialize, Deserialize, Debug, FromRow, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
}

#[cfg(test)]

impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
            password_hash: None,
        }
    }
}
