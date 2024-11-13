use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;

use crate::{
    models::{CreateChat, UpdateChat},
    AppError, AppState,
};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_chats(user.ws_id).await?;
    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn update_chat_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = match id.parse::<i64>() {
        Ok(id) => state.update_chat_by_id(id, input).await?,
        Err(_) => return Err(AppError::UpdateChatError("Invalid chat id".to_string())),
    };

    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    match id.parse::<i64>() {
        Ok(id) => state.delete_chat_by_id(id).await?,
        Err(_) => return Err(AppError::UpdateChatError("Invalid chat id".to_string())),
    };

    Ok(StatusCode::NO_CONTENT)
}
