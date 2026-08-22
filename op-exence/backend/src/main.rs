use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = op_exence::config::Config::from_env();

    if let Err(err) = op_exence::app::run(config).await {
        tracing::error!(error = %err, "server failed");
        std::process::exit(1);
    }
}
