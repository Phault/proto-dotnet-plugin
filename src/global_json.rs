use std::ops::Mul;

use proto_pdk::{UnresolvedVersionSpec, Version};
use semver::Error;
use serde::Deserialize;

/// Represents `global.json`
#[derive(Deserialize)]
pub struct GlobalJson {
    pub sdk: Option<GlobalJsonSdk>,
}

#[derive(Deserialize, Default)]
pub struct GlobalJsonSdk {
    pub version: Option<String>,

    #[serde(rename = "allowPrerelease")]
    pub allow_prerelease: Option<bool>,

    #[serde(rename = "rollForward")]
    pub roll_forward: Option<RollForward>,
}

#[derive(Deserialize, Clone)]
pub enum RollForward {
    #[serde(rename = "major")]
    Major,
    #[serde(rename = "minor")]
    Minor,
    #[serde(rename = "feature")]
    Feature,
    #[serde(rename = "patch")]
    Patch,
    #[serde(rename = "latestMajor")]
    LatestMajor,
    #[serde(rename = "latestMinor")]
    LatestMinor,
    #[serde(rename = "latestFeature")]
    LatestFeature,
    #[serde(rename = "latestPatch")]
    LatestPatch,
    #[serde(rename = "disable")]
    Disable,
}

impl GlobalJsonSdk {
    pub fn to_version_spec(&self) -> Result<UnresolvedVersionSpec, Error> {
        let roll_forward = if self.version.is_some() {
            self.roll_forward
                .clone()
                .unwrap_or(RollForward::LatestPatch)
        } else {
            RollForward::LatestMajor
        };

        let min_version = self.version.clone().unwrap_or("0.0.0".into());

        // proto does not support this earliest matching, it will always pick the latest you allow it
        match roll_forward {
            RollForward::LatestMajor | RollForward::Major => {
                UnresolvedVersionSpec::parse(format!(">={}", min_version))
            }
            RollForward::LatestMinor | RollForward::Minor => {
                UnresolvedVersionSpec::parse(format!("^{}", min_version))
            }
            RollForward::LatestFeature | RollForward::Feature => {
                UnresolvedVersionSpec::parse(format!("~{}", min_version))
            }
            RollForward::LatestPatch | RollForward::Patch => {
                let max_version = Version::parse(&min_version)?;
                let max_version = format!(
                    "{}.{}.{}",
                    max_version.major,
                    max_version.minor,
                    // in 0.0.xyy x is the feature, yy is the patch
                    // hence we round semver patch to the next hundredth
                    max_version.patch.div_ceil(100).mul(100)
                );

                UnresolvedVersionSpec::parse(format!(">={min_version} <{max_version}"))
            }
            RollForward::Disable => UnresolvedVersionSpec::parse(min_version),
        }
    }
}
