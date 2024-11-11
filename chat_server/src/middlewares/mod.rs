mod auth;
mod request_id;
mod server_time;

pub use auth::verify_token;
use axum::middleware::from_fn;
use request_id::set_request_id;
use server_time::SeverTimeLayer;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";

pub fn set_layer(router: Router) -> Router {
    router.layer(
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
