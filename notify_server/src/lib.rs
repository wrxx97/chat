mod config;
mod error;
mod notify;
mod sse;

use anyhow::Result;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{
    middlewares::{verify_token, TokenVerify},
    utils::DecodingKey,
};
use dashmap::DashMap;
use notify::ChatEvent;
use sse::sse_handler;
use std::{ops::Deref, sync::Arc};
use tokio::sync::broadcast;

pub use config::AppConfig;
pub use error::AppError;
pub use notify::setup_pg_listener;
pub type UserMap = Arc<DashMap<i64, broadcast::Sender<Arc<ChatEvent>>>>;

const INDEX_HTML: &str = include_str!("../index.html");

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub config: AppConfig,
    pub users: UserMap,
    pub dk: DecodingKey,
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<chat_core::User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).expect("load pk failed");
        let users = Arc::new(DashMap::new());

        Self {
            inner: Arc::new(AppStateInner { dk, config, users }),
        }
    }
}
