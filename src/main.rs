use color_eyre::eyre::eyre;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let url = get_cat_image_url().await.unwrap();
    println!("The cat image is at {url}")
}

async fn get_cat_image_url() -> color_eyre::Result<String> {
    let api_url = "https://api.thecatapi.com/v1/images/search";
    let response = reqwest::get(api_url).await.unwrap();
    if !response.status().is_success() {
        return Err(eyre!(
            "Request failed with HTTP status {}",
            response.status()
        ));
    }

    let mut images: Vec<CatImage> = response.json().await?;

    if let Some(image) = images.pop() {
        Ok(image.url)
    } else {
        Err(eyre!("The Cat API returned no images"))
    }
}

#[derive(Deserialize)]
struct CatImage {
    url: String,
}
