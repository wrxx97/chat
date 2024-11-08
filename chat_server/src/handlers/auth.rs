use axum::{http::StatusCode, response::IntoResponse};

pub(crate) async fn signin_handler() -> impl IntoResponse {
    (StatusCode::OK, "Signin")
}

pub(crate) async fn signup_handler() -> impl IntoResponse {
    (StatusCode::OK, "Signup")
}
