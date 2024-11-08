mod config;
mod error;
mod handlers;
mod models;

use axum::routing::{get, patch, post};
use axum::Router;
use std::ops::Deref;
use std::sync::Arc;

pub use config::*;
pub use error::AppError;
use handlers::*;
pub use models::User;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);

    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler).delete(delete_chat_handler),
        )
        .route(
            "/chat/:id/messages",
            get(list_msg_handler).post(sned_msg_handler),
        )
        .with_state(state.clone());

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state)
}

// state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}
