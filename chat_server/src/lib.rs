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
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler).delete(delete_chat_handler),
        )
        .route(
            "/chat/:id/messages",
            get(list_msg_handler).post(sned_msg_handler),
        )
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
        let server_url = config.server.db_url.clone();

        let tdb = sqlx_db_tester::TestPg::new(
            server_url.to_string(),
            std::path::Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;
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
