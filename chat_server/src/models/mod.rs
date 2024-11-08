mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
}
