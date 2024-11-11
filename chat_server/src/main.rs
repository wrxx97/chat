use anyhow::Result;
use chat_server::{get_router, AppConfig};
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

    let config = AppConfig::load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {:?}", addr);

    let app = get_router(config).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
