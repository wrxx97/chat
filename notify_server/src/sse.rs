use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Extension,
};
use axum_extra::{headers, TypedHeader};
use chat_core::User;
use futures::stream::Stream;
use jwt_simple::reexports::serde_json;
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tracing::info;

use crate::{AppState, ChatEvent};

const CHANNEL_CAPACITY: usize = 256;

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("`{}` connected", user_agent.as_str());

    let user_id = user.id;
    let users = &state.users;

    let rx = if let Some(tx) = users.get(&user_id) {
        info!("user {} reconnected", user_id);
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        users.insert(user_id, tx);
        rx
    };

    let stream = BroadcastStream::new(rx).filter_map(|v| {
        if let Ok(event) = v {
            let name = match *event {
                ChatEvent::NewChat(_) => "NewChat",
                ChatEvent::AddToChat(_) => "AddToChat",
                ChatEvent::RemoveFromChat(_) => "RemoveFromChat",
                ChatEvent::NewMessage(_) => "NewMessage",
            };
            let data = serde_json::to_string(&event).expect("serialize event failed");
            Some(Ok(Event::default().data(data).event(name)))
        } else {
            None
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
