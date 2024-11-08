use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("argon2 error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),
}
