use extism_pdk::Error;
use proto_pdk::fetch_url;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DotnetReleaseSdkFile {
    pub name: String,

    /// Always available for standard builds, but some older builds targeting rhel (or "win-gs" whatever that is), will omit this field.
    pub rid: Option<String>,

    pub url: String,
    pub hash: String,
}

#[derive(Deserialize, Clone)]
pub struct DotnetReleaseSdk {
    /// Examples:
    /// - 8.0.100
    /// - 8.0.100-rc.2.23502.2
    /// - 8.0.0-preview.7.23375.6
    pub version: String,

    /// Mostly the same as `version`, except for prereleases
    ///
    /// Examples:
    /// - 8.0.100
    /// - 8.0.100-rc.2
    /// - 8.0.0-preview.7
    #[serde(rename = "version-display")]
    pub version_display: Option<String>,

    pub files: Vec<DotnetReleaseSdkFile>,
}

#[derive(Deserialize, Clone)]
pub struct DotnetRelease {
    /// Latest sdk for this runtime version, usually represented in the `sdks` field as well.
    pub sdk: DotnetReleaseSdk,

    /// All sdk versions for this runtime version.
    /// Can be None, if so fallback to the `sdk` field.
    pub sdks: Option<Vec<DotnetReleaseSdk>>,
}

#[derive(Deserialize, Clone)]
pub struct DotnetReleasesJson {
    pub releases: Vec<DotnetRelease>,
}

#[derive(Deserialize, Clone)]
pub struct DotnetReleasesIndex {
    #[serde(rename = "channel-version")]
    pub channel_version: String,

    #[serde(rename = "latest-sdk")]
    pub latest_sdk: String,

    #[serde(rename = "release-type")]
    pub release_type: String,

    #[serde(rename = "releases.json")]
    pub releases_json: String,
}

#[derive(Deserialize, Clone)]
pub struct DotnetReleasesIndexJson {
    #[serde(rename = "releases-index")]
    pub releases_index: Vec<DotnetReleasesIndex>,
}

pub fn fetch_release_index() -> Result<Vec<DotnetReleasesIndex>, Error> {
    fetch_url("https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json")
        .map(|r: DotnetReleasesIndexJson| r.releases_index)
        .map_err(|e| e.context(format!("Failed to retrieve index of releases")))
}
