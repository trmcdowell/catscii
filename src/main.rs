use axum::{
    extract::{MatchedPath, Request},
    routing::get,
    Router,
};
use catscii::{analytics_get, health_check, locat::Locat, root_get, ServerState};
use dotenv::dotenv;
use reqwest::Method;
use std::{str::FromStr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, info_span, warn, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build tracing subscriber
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    // Set up application
    let shutdown_signal = async {
        _ = tokio::signal::ctrl_c().await;
        warn!("Initiating graceful shutdown");
    };

    dotenv().ok();
    let address = std::env::var("ADDRESS").expect("$ADDRESS must be set");
    let country_db_path =
        std::env::var("GEOLITE2_COUNTRY_DB").expect("$GEOLITE2_COUNTRY_DB must be set");
    let analytics_db_path = std::env::var("ANALYTICS_DB").expect("$ANALYTICS_DB must be set");

    let state = ServerState {
        client: Default::default(),
        locat: Arc::new(Locat::new(&country_db_path, &analytics_db_path).unwrap()),
    };
    let app: Router = Router::new()
        .route("/", get(root_get))
        .route("/analytics", get(analytics_get))
        .route("/health_check", get(health_check))
        .layer(CorsLayer::new().allow_methods([Method::GET]))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);
                let request_id = Uuid::new_v4();

                info_span!(
                    "http_request",
                    request_id = tracing::field::display(request_id),
                    method = ?request.method(),
                    matched_path,
                )
            }),
        )
        .with_state(state);

    let listener = TcpListener::bind(address).await?;
    info!("Listening on {:?}", listener.local_addr().unwrap());

    // Run app
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}
