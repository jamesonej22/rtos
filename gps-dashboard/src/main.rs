mod api;
mod gps;

use tokio::sync::watch;
use tracing_subscriber::EnvFilter;

use gps::{GpsTelemetry, run_gpsd_client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let (telemetry_tx, telemetry_rx) = watch::channel(GpsTelemetry::default());

    tokio::spawn(async move {
        run_gpsd_client(telemetry_tx).await;
    });

    api::run_api_server(telemetry_rx).await?;

    Ok(())
}
