use std::sync::Arc;
use std::{fmt::Write, net::IpAddr};

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response},
    response::IntoResponse,
};
use locat::Locat;
use reqwest::{header, Client, StatusCode};
use tracing::{error, info, warn};

pub mod locat;

#[derive(Clone)]
pub struct ServerState {
    pub client: reqwest::Client,
    pub locat: Arc<Locat>,
}

pub async fn health_check() -> Response<Body> {
    (StatusCode::OK, "Application is healthy").into_response()
}

pub async fn analytics_get(State(state): State<ServerState>) -> Response<Body> {
    let analytics = state.locat.get_analytics().await.unwrap();
    let mut response = String::new();
    for (country, count) in analytics {
        _ = writeln!(&mut response, "{country}: {count}");
    }
    (StatusCode::OK, response).into_response()
}

pub async fn root_get(headers: HeaderMap, State(state): State<ServerState>) -> Response<Body> {
    if let Some(addr) = get_client_addr(headers) {
        match state.locat.ip_to_iso_code(addr) {
            Some(country) => {
                info!("Got request from {country}");
            }
            None => warn!("Could not determine country for request"),
        }
    }

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

fn get_client_addr(headers: HeaderMap) -> Option<IpAddr> {
    let header = headers.get("fly-client-ip")?;
    let header = header.to_str().ok()?;
    let addr = header.parse::<IpAddr>().ok()?;
    Some(addr)
}

async fn create_cat_ascii_art(state: ServerState) -> anyhow::Result<String> {
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
