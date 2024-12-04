use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{AppError, AppState};

pub async fn get_user_list_handler(
    Path(_ws_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let ws_id: i32 = _ws_id
        .parse()
        .map_err(|_| AppError::NotFound("not found workspace".to_string()))?;
    let users = state.fetch_all_users(ws_id).await?;
    Ok((StatusCode::OK, Json(users)))
}
