mod sse;

use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{Chat, Message};
use futures::StreamExt;
use sqlx::postgres::PgListener;
use sse::sse_handler;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/events", get(sse_handler))
        .route("/", get(index_handler))
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener() -> Result<()> {
    let mut listener = PgListener::connect("postgresql://postgres:postgres@localhost:5432").await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            println!("Received notification: {:#?}", notification);
        }
    });

    Ok(())
}
