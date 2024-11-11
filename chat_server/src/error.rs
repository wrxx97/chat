use axum::{body::Body, http::status, response::IntoResponse, Json};
use jwt_simple::reexports::serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("argon2 error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::http::Response<Body> {
        let status = match self {
            AppError::SqlxError(_) => status::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Argon2Error(_) => status::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtError(_) => status::StatusCode::FORBIDDEN,
            AppError::EmailAlreadyExists(_) => status::StatusCode::CONFLICT,
        };

        (
            status,
            Json(json!({
                "error":self.to_string(),
            })),
        )
            .into_response()
    }
}
