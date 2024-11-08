use std::mem;

use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

impl User {
    pub async fn find_by_email(email: &str, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, fullname, email, created_at FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn create(
        email: &str,
        password: &str,
        fullname: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;

        let user = sqlx::query_as(
            r#"
            INSERT INTO users (email, password_hash, fullname)
            VALUES ($1, $2, $3)
            RETURNING id, fullname, email, created_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(fullname)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify(
        email: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, fullname, email, created_at, password_hash FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        if let Some(mut user) = user {
            let password_hash = mem::take(&mut user.password_hash).unwrap_or_default();
            let is_valid = verify_password(password, &password_hash)?;

            if is_valid {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, parsed_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();

    // Parse stored hash back to PasswordHash struct
    let parsed_hash = PasswordHash::new(parsed_hash)?;

    // Verify password
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use sqlx_db_tester::TestPg;

    use super::*;

    #[tokio::test]
    async fn test_hash_password() {
        let password = "password";
        let password_hash = hash_password(password).unwrap();

        assert_ne!(password, password_hash);
    }

    #[tokio::test]
    async fn test_verify_password() {
        let password = "password";
        let password_hash = hash_password(password).unwrap();

        assert!(verify_password(password, &password_hash).unwrap());
    }

    #[tokio::test]
    async fn test_user_create_find_verify() -> Result<(), AppError> {
        let tdb = TestPg::new(
            "postgres://postgres:101350@localhost:5432".to_string(),
            std::path::Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        // do something with the pool

        let email = "wrxx@gmail.com";
        let password = "password";
        let fullname = "wrxx";

        let user = User::create(email, password, fullname, &pool).await?;

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        let user = User::verify(email, password, &pool).await?;
        assert!(user.is_some());

        Ok(())

        // when tdb gets dropped, the database will be dropped
    }
}
