use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

use super::REQUEST_ID_HEADER;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if x-request-id exists, do nothing, otherwise generate a new one
    let req_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let request_id = uuid::Uuid::new_v4().to_string();
            match request_id.parse::<HeaderValue>() {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;

    if let Some(id) = req_id {
        res.headers_mut().insert(REQUEST_ID_HEADER, id);
    }

    res
}
