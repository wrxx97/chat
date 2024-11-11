use axum::{http::StatusCode, response::IntoResponse, Extension};
use tracing::info;

use crate::User;

pub(crate) async fn list_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
    info!("user: {:?}", user);
    (StatusCode::OK, "List chat")
}

pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    (StatusCode::OK, "Create chat")
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    (StatusCode::OK, "Update chat")
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    (StatusCode::OK, "Delete chat")
}
