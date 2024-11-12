use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;
use tracing::warn;

use crate::{models::ChatFile, AppError, AppState, User};

pub(crate) async fn send_msg_handler() -> impl IntoResponse {
    (StatusCode::OK, "send messages")
}

pub(crate) async fn list_msg_handler() -> impl IntoResponse {
    (StatusCode::OK, "List messages")
}

pub(crate) async fn upload_file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = &state.config.server.base_dir;
    let mut files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().map(|s| s.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart field");
            continue;
        };

        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(base_dir);

        if path.exists() {
            warn!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }

    Ok((StatusCode::OK, Json(files)))
}

pub(crate) async fn download_file_handler(
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    let base_dir = &state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);

    if !path.exists() {
        return Err(AppError::NotFound("file not found".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    // TODO: streaming
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());
    Ok((headers, body))
}
