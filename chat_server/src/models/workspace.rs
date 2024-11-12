use sqlx::PgPool;

use crate::AppError;

use super::{ChatUser, Workspace};

impl Workspace {
    pub async fn create(name: &str, user_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
        INSERT INTO workspaces (name, owner_id)
        VALUES ($1, $2)
        RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        // update owner_id in two cases 1) owner_id = 0 2) owner's ws_id = id
        let ws = sqlx::query_as(
            r#"
        UPDATE workspaces
        SET owner_id = $1
        WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(owner_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    #[allow(unused)]
    pub async fn fetch_all_chat_users(
        ws_id: i64,
        pool: &PgPool,
    ) -> Result<Vec<ChatUser>, AppError> {
        let users: Vec<ChatUser> = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1 order by id
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    #[allow(unused)]
    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws: Option<Workspace> = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(unused)]
    pub async fn find_by_id(id: i64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws: Option<Workspace> = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        get_test_pool,
        models::{CreateUser, User},
    };
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::create("test", 0, &pool).await?;

        let input = CreateUser::new(&ws.name, "wrx", "wrxx@qq.com", "wrxx");
        let user = User::create(&input, &pool).await?;

        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &pool).await?;
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::find_by_name("acme", &pool).await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 5);

        Ok(())
    }
}
