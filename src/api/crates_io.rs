use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CratesIoResponse {
    #[serde(rename = "crate")]
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

    crate::api::retry(
        || async {
            let response = client
                .get(&url)
                .header(
                    "User-Agent",
                    "cargo-diagnose (github.com/anaslimem/cargo-diagnose)",
                )
                .send()
                .await?;

            if response.status().is_success() {
                let res: CratesIoResponse = response.json().await?;
                Ok(res)
            } else {
                Err(format!("Crates.io API error: {}", response.status()).into())
            }
        },
        3,
    )
    .await
}
