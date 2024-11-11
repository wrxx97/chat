use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tracing::warn;

use crate::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let token = TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await;

    let msg = match token {
        Ok(TypedHeader(Authorization(v))) => {
            let token = v.token();
            match state.dk.verify(token) {
                Ok(user) => {
                    let mut req = Request::from_parts(parts, body);
                    req.extensions_mut().insert(user);
                    return next.run(req).await;
                }
                Err(e) => {
                    format!("Failed to verify token: {}", e)
                }
            }
        }
        Err(e) => {
            format!("Failed to extract token: {}", e)
        }
    };

    warn!(msg);
    (StatusCode::UNAUTHORIZED, msg).into_response()
}
