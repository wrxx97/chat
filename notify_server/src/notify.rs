use std::{collections::HashSet, sync::Arc};

use crate::AppState;
use anyhow::Result;
use chat_core::{Chat, Message};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

#[derive(Debug)]
struct Notification {
    // users being impacted, so we should send the event to them
    user_ids: HashSet<i64>,
    event: Arc<ChatEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdater {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    #[serde(flatten)]
    message: Message,
    #[serde(flatten)]
    chat: Chat,
}

pub async fn setup_pg_listener(state: AppState) -> Result<()> {
    let mut listener = PgListener::connect(&state.config.server.db_url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notif)) = stream.next().await {
            info!("Received notification: {:?}", notif);
            let notification = Notification::load(notif.channel(), notif.payload())?;
            let users = &state.users;
            for user_id in &notification.user_ids {
                if let Some(tx) = users.get(user_id) {
                    if let Err(e) = tx.send(notification.event.clone()) {
                        warn!("send to user {} failed: {}", user_id, e);
                    }
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}

impl Notification {
    fn load(r#type: &str, payload: &str) -> Result<Self> {
        match r#type {
            "chat_updated" => {
                let updater: ChatUpdater = serde_json::from_str(payload)?;
                let user_ids = get_affected_user_ids(updater.old.as_ref(), updater.new.as_ref());
                let event = match updater.op.as_str() {
                    "INSERT" => ChatEvent::NewChat(updater.new.expect("new should exist")),
                    "UPDATE" => ChatEvent::AddToChat(updater.new.expect("update should exist")),
                    "DELETE" => ChatEvent::RemoveFromChat(updater.old.expect("old should exist")),
                    _ => return Err(anyhow::anyhow!("Invalid op")),
                };
                Ok(Self {
                    user_ids,
                    event: Arc::new(event),
                })
            }
            "chat_message_created" => {
                let message: ChatMessageCreated = serde_json::from_str(payload)?;
                let user_ids = message.chat.members.iter().cloned().collect();
                Ok(Self {
                    user_ids,
                    event: Arc::new(ChatEvent::NewMessage(message.message)),
                })
            }
            _ => Err(anyhow::anyhow!("Invalid notification type")),
        }
    }
}

fn get_affected_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<i64> {
    let mut user_ids = HashSet::new();
    match (old, new) {
        (Some(old), Some(new)) => {
            user_ids.extend(old.members.iter());
            user_ids.extend(new.members.iter());
        }
        (Some(old), None) => {
            user_ids.extend(old.members.iter());
        }
        (None, Some(new)) => {
            user_ids.extend(new.members.iter());
        }
        _ => {}
    }

    user_ids
}
