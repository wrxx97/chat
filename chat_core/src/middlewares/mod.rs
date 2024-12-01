mod auth;
mod request_id;
mod server_time;

use std::fmt;

pub use auth::verify_token;
use axum::middleware::from_fn;
use request_id::set_request_id;
use server_time::SeverTimeLayer;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

use crate::User;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";

pub fn set_layer(router: Router) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any) // 允许所有来源（或指定具体来源）
        .allow_methods(Any) // 允许所有方法（GET、POST 等）
        .allow_headers(Any); // 允许所有请求头

    router.layer(cors).layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(SeverTimeLayer),
    )
}

pub trait TokenVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}
