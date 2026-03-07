use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CratesIoResponse {
    pub crate_data: CrateData,
}

#[derive(Deserialize)]
pub struct CrateData {
    pub max_version: String,
    pub repository: Option<String>,
}

pub async fn get_crate_info(
    client: &Client,
    name: &str,
) -> Result<CratesIoResponse, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://crates.io/api/v1/crates/{}", name);

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "cargo-doctor (github.com/manaslimem/cargo-doctor)",
        )
        .send()
        .await?;

    if response.status().is_success() {
        // Crates.io API nests the core data under {"crate": {...}}
        // Because `crate` is a reserved keyword in Rust, the json response must map it.
        // We'll parse the raw string then map it manually or use a wrapper.

        #[derive(Deserialize)]
        struct RawResponse {
            #[serde(rename = "crate")]
            inner: CrateData,
        }

        let raw: RawResponse = response.json().await?;
        Ok(CratesIoResponse {
            crate_data: raw.inner,
        })
    } else {
        Err(format!("Crates.io API error: {}", response.status()).into())
    }
}
