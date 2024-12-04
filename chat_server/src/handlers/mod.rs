mod auth;
mod chat;
mod msgs;
mod user;

use axum::{http::StatusCode, response::IntoResponse};

pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use msgs::*;
pub(crate) use user::*;

pub(crate) async fn index_handler() -> impl IntoResponse {
    (StatusCode::OK, "Hello, World!")
}
