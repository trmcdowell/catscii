use std::str::FromStr;

use anyhow::anyhow;
use tokio::net::TcpListener;

use axum::{
    body::Body,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use dotenv::dotenv;
use reqwest::{header, StatusCode};
use serde::Deserialize;
use tracing::{error, info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

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

    dotenv().ok();
    let address = std::env::var("ADDRESS")?;
    let app: Router = Router::new().route("/", get(root_get));
    let listener = TcpListener::bind(address).await?;
    info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn root_get() -> Response<Body> {
    match get_cat_ascii_art().await {
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

async fn get_cat_ascii_art() -> anyhow::Result<String> {
    let api_url = "https://api.thecatapi.com/v1/images/search";
    let client = reqwest::Client::default();

    let image = client
        .get(api_url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<CatImage>>()
        .await?
        .pop()
        .ok_or_else(|| anyhow!("The Cat API returned no images"))?;
    println!("{}", image.url);

    let image_bytes = client
        .get(image.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let image = image::load_from_memory(&image_bytes)?;
    let artem_config = artem::config::ConfigBuilder::new()
        .target(artem::config::TargetType::HtmlFile)
        .build();
    let ascii = artem::convert(image, &artem_config);

    Ok(ascii)
}

#[derive(Deserialize)]
struct CatImage {
    url: String,
}
