use anyhow::Result;
use notify_server::{get_router, setup_pg_listener, AppConfig, AppState};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    self, fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:6687".to_string();
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {:?}", addr);

    let config = AppConfig::load()?;
    let state = AppState::new(config);

    let app = get_router(state.clone());
    setup_pg_listener(state).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
