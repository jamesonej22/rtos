use std::convert::Infallible;

use axum::{
    Router,
    extract::State,
    response::{
        Json,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use tokio::sync::watch;
use tower_http::services::ServeDir;

use crate::gps::GpsTelemetry;

#[derive(Clone)]
struct AppState {
    telemetry: watch::Receiver<GpsTelemetry>,
}

async fn get_gps(State(state): State<AppState>) -> Json<GpsTelemetry> {
    Json(state.telemetry.borrow().clone())
}

async fn gps_stream(
    State(state): State<AppState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let mut receiver = state.telemetry;

    let stream = async_stream::stream! {
        // Send the current state and mark it as seen.
        let initial = receiver.borrow_and_update().clone();

        if let Ok(json) = serde_json::to_string(&initial) {
            yield Ok(Event::default().data(json));
        }

        // Send subsequent changes.
        loop {
            if receiver.changed().await.is_err() {
                break;
            }

            let telemetry = receiver.borrow_and_update().clone();

            if let Ok(json) = serde_json::to_string(&telemetry) {
                yield Ok(Event::default().data(json));
            }
        }
    };

    Sse::new(stream).keep_alive(
        KeepAlive::default()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn run_api_server(
    telemetry_rx: watch::Receiver<GpsTelemetry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        telemetry: telemetry_rx,
    };

    let static_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/static");

    let app = Router::new()
        .route("/api/gps", get(get_gps))
        .route("/api/gps/stream", get(gps_stream))
        .fallback_service(ServeDir::new(static_dir))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    tracing::info!("GPS dashboard listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
