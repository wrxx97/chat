use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    models::{Chat, CreateChat, UpdateChat},
    AppError, AppState, User,
};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = Chat::fetch_all(user.ws_id, &state.pg_pool).await?;
    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(input, user.ws_id, &state.pg_pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn update_chat_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = match id.parse::<i64>() {
        Ok(id) => Chat::update_by_id(id, input, &state.pg_pool).await?,
        Err(_) => return Err(AppError::UpdateChatError("Invalid chat id".to_string())),
    };

    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    match id.parse::<i64>() {
        Ok(id) => Chat::delete_by_id(id, &state.pg_pool).await?,
        Err(_) => return Err(AppError::UpdateChatError("Invalid chat id".to_string())),
    };

    Ok(StatusCode::NO_CONTENT)
}
