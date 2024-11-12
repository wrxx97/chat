use std::mem;

use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Deserialize;
use sqlx::PgPool;

use super::{ChatUser, Workspace};

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub fullname: String,
    pub workspace: String,
}

#[derive(Deserialize)]
pub struct SignInUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    // TODO: use transaction for workspace creation and user creation
    pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        // check if email exists
        let user = Self::find_by_email(&input.email, pool).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match Workspace::find_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, 0, pool).await?,
        };

        let password_hash = hash_password(&input.password)?;
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (ws_id, email, fullname, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(ws.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id as _, pool).await?;
        }

        Ok(user)
    }

    pub async fn verify(input: SignInUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let email = &input.email;
        let password = &input.password;
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at, password_hash FROM users WHERE email = $1
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

    pub async fn add_to_workspace(&self, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let user: Self = sqlx::query_as(
            r#"
            UPDATE users
            SET ws_id = $1
            WHERE id = $2
            "#,
        )
        .bind(ws_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}

impl ChatUser {
    pub async fn fetch_by_ids(ids: &[i64], pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as(r#"SELECT * FROM users WHERE id = ANY($1)"#)
            .bind(ids)
            .fetch_all(pool)
            .await?;

        Ok(users)
    }
    #[allow(dead_code)]
    pub async fn fetch_all(ws_id: i64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(users)
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
impl CreateUser {
    pub fn new(workspace: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            workspace: workspace.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            fullname: fullname.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_pool;

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
        let (_tdb, pool) = get_test_pool(None).await;

        let email = "wrxx@gmail.com";
        let password = "password";
        let fullname = "wrxx";
        let input = CreateUser::new("default", fullname, email, password);
        let user = User::create(&input, &pool).await?;

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        let input = SignInUser {
            email: email.to_string(),
            password: password.to_string(),
        };
        let user = User::verify(input, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_ids() -> Result<(), AppError> {
        let (_tdb, pool) = get_test_pool(None).await;

        let users = ChatUser::fetch_by_ids(&[1, 2], &pool).await?;
        assert_eq!(users.len(), 2);

        Ok(())
    }
}
