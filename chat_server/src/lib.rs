mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use axum::middleware::from_fn_with_state;
use axum::routing::{get, patch, post};
use axum::Router;
use core::fmt;
use middlewares::{set_layer, verify_token};
use sqlx::PgPool;
use tokio::fs;

use std::ops::Deref;
use std::sync::Arc;
use utils::{DecodingKey, EncodingKey};

pub use config::*;
pub use error::AppError;
use handlers::*;
pub use models::User;

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
        .layer(from_fn_with_state(state.clone(), verify_token))
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
        let dk = DecodingKey::load_pem(&config.auth.sk).expect("load pk failed");
        let ek = EncodingKey::load_pem(&config.auth.pk).expect("load sk failed");
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

#[cfg(test)]

impl AppState {
    pub async fn new_for_test() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        let config = AppConfig::load()?;
        let dk = DecodingKey::load_pem(&config.auth.sk).expect("load pk failed");
        let ek = EncodingKey::load_pem(&config.auth.pk).expect("load sk failed");

        let (tdb, pool) = get_test_pool(Some(&config.server.db_url)).await;

        Ok((
            tdb,
            Self {
                inner: Arc::new(AppStateInner {
                    config,
                    dk,
                    ek,
                    pg_pool: pool,
                }),
            },
        ))
    }
}

#[cfg(test)]
pub async fn get_test_pool(url: Option<&str>) -> (sqlx_db_tester::TestPg, PgPool) {
    use sqlx::Executor;

    let url = url.unwrap_or("postgres://postgres:postgres@localhost:5432/");
    let post = url.rfind('/').expect("invalid db_url");
    let server_url = &url[..post];

    let tdb = sqlx_db_tester::TestPg::new(
        server_url.to_string(),
        std::path::Path::new("../migrations"),
    );
    let pool = tdb.get_pool().await;

    // run prepared test data
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
