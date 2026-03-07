use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OsvQuery {
    version: String,
    package: OsvPackage,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Deserialize)]
pub struct OsvResponse {
    pub vulns: Option<Vec<OsvVulnerability>>,
}

#[derive(Deserialize)]
pub struct OsvVulnerability {
    pub id: String,
}

pub async fn check_vulnerabilities(
    client: &Client,
    name: &str,
    version: &str,
) -> Result<OsvResponse, Box<dyn std::error::Error + Send + Sync>> {
    crate::api::retry(
        || async {
            let query = OsvQuery {
                version: version.to_string(),
                package: OsvPackage {
                    name: name.to_string(),
                    ecosystem: "crates.io".to_string(),
                },
            };

            let response = client
                .post("https://api.osv.dev/v1/query")
                .json(&query)
                .send()
                .await?;

            let osv_response: OsvResponse = response.json().await?;
            Ok(osv_response)
        },
        3,
    )
    .await
}
