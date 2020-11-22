use beatsaver_rs::{map::Map, Page};
use serde_json;
use std::error::Error;
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let data = reqwest::get("https://beatsaver.com/api/maps/rating")
        .await?.text().await?;

    let page: Page<Map> = serde_json::from_str(data.as_str())?;

    for i in page.docs {
        println!("Song: \"{}\"", i.name);
    }

    Ok(())
}
