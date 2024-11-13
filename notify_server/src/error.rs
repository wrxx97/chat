use axum::{body::Body, http::status, response::IntoResponse, Json};
use jwt_simple::reexports::serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("std error: {0}")]
    StdError(#[from] std::io::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::http::Response<Body> {
        let status = match self {
            AppError::StdError(_) => status::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtError(_) => status::StatusCode::FORBIDDEN,
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
