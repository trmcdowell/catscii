use axum::{
    body::Body,
    extract::{MatchedPath, Request, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use dotenv::dotenv;
use reqwest::{header, Client, Method, StatusCode};
use std::str::FromStr;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, info_span, warn, Level};
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
    let address = std::env::var("ADDRESS")?;
    let state = AppState::default();
    let app: Router = Router::new()
        .route("/", get(get_cat_ascii_art))
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

async fn get_cat_ascii_art(State(state): State<AppState>) -> Response<Body> {
    match create_cat_ascii_art(state).await {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            error!("Error getting ascii art: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Error getting ascii art").into_response()
        }
    }
}

async fn create_cat_ascii_art(state: AppState) -> anyhow::Result<String> {
    let image_bytes = download_file(&state.client, "https://cataas.com/cat").await?;
    let image = image::load_from_memory(&image_bytes)?;
    let artem_config = artem::config::ConfigBuilder::new()
        .target(artem::config::TargetType::HtmlFile)
        .build();
    let ascii = artem::convert(image, &artem_config);

    Ok(ascii)
}

async fn download_file(client: &Client, url: &str) -> anyhow::Result<Vec<u8>> {
    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes.to_vec())
}

async fn health_check() -> Response<Body> {
    (StatusCode::OK, "Application is healthy").into_response()
}

#[derive(Clone, Default)]
struct AppState {
    client: reqwest::Client,
}
