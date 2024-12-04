mod config;
mod error;
mod handlers;
mod models;

use axum::middleware::from_fn_with_state;
use axum::routing::{get, patch, post};
use axum::Router;
use chat_core::middlewares::{set_layer, verify_token, TokenVerify};
use chat_core::utils::{DecodingKey, EncodingKey};
use core::fmt;
use sqlx::PgPool;
use tokio::fs;

use std::ops::Deref;
use std::sync::Arc;

pub use config::*;
pub use error::AppError;
use handlers::*;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pg_pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .with_state(state.clone())
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/:id",
            patch(update_chat_handler).delete(delete_chat_handler),
        )
        .route(
            "/chats/:id/messages",
            get(list_msg_handler).post(send_msg_handler),
        )
        .route("/files/upload", post(upload_file_handler))
        .route("/files/:ws_id/*path", get(download_file_handler))
        .route("/users/:ws_id", get(get_user_list_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let router = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(router))
}

// state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir).await?;
        let dk = DecodingKey::load(&config.auth.pk).expect("load pk failed");
        let ek = EncodingKey::load(&config.auth.sk).expect("load sk failed");
        let pg_pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("connect to db failed");
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pg_pool,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<chat_core::User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = AppConfig::load()?;

            let dk = DecodingKey::load(&config.auth.pk).expect("load pk failed");
            let ek = EncodingKey::load(&config.auth.sk).expect("load sk failed");
            let post = config.server.db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[..post];
            let (tdb, pg_pool) = get_test_pool(Some(server_url)).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pg_pool,
                }),
            };
            Ok((tdb, state))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:postgres@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }
}
