use serde::Deserialize;

/// Represents `global.json`
#[derive(Deserialize)]
pub struct GlobalJson {
    pub sdk: Option<GlobalJsonSdk>,
}

#[derive(Deserialize)]
pub struct GlobalJsonSdk {
    pub version: Option<String>,

    #[serde(rename = "allowPrerelease")]
    pub allow_prerelease: Option<bool>,

    #[serde(rename = "rollForward")]
    pub roll_forward: Option<String>,
}
