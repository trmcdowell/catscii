use serde::Deserialize;

#[tokio::main]
async fn main() {
    let response = reqwest::get("https://api.thecatapi.com/v1/images/search")
        .await
        .unwrap();
    if !response.status().is_success() {
        panic!("Request failed with HTTP status {}", response.status());
    }

    println!("Status: {}", response.status());
    let images: Vec<CatImage> = response.json().await.unwrap();
    let image = images
        .first()
        .expect("Cat API should return at least one image");
    println!("The image is at {}", image.url);
}

#[derive(Deserialize)]
struct CatImage {
    url: String,
}
