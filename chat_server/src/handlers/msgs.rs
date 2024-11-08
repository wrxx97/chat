use axum::{http::StatusCode, response::IntoResponse};

pub(crate) async fn sned_msg_handler() -> impl IntoResponse {
    (StatusCode::OK, "send messages")
}

pub(crate) async fn list_msg_handler() -> impl IntoResponse {
    (StatusCode::OK, "List messages")
}
