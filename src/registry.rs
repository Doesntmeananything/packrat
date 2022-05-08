//! Interact with the NPM registry to get up-to-date package version information.
//!
//! For reference, see [official NPM registry documentation](https://github.com/npm/registry/blob/master/docs/responses/package-metadata.md).

use reqwest::{header::ACCEPT, Client};
use serde::Deserialize;

/// Registry metadata of an NPM package.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Metadata {
    pub name: String,
    pub dist_tags: LatestVersion,
}

#[derive(Deserialize, Debug)]
pub struct LatestVersion {
    pub latest: String,
}

/// NPM registry base URL.
const REGISTRY_URL: &str = "https://registry.npmjs.org/";
/// `ACCEPT` header that signals to registry to respond with metadata in abbreviated form.
const ACCEPT_ABBREVIATED: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

pub async fn fetch_metadata(
    client: &Client,
    package_name: &str,
) -> Result<Metadata, reqwest::Error> {
    client
        .get(REGISTRY_URL.to_owned() + package_name)
        .header(ACCEPT, ACCEPT_ABBREVIATED)
        .send()
        .await?
        .json::<Metadata>()
        .await
}
