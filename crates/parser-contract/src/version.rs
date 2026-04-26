use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

pub const CURRENT_CONTRACT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct ContractVersion {
    pub semver: Version,
}

impl ContractVersion {
    pub fn current() -> Self {
        Self::parse(CURRENT_CONTRACT_VERSION)
            .expect("CURRENT_CONTRACT_VERSION must be a valid semantic version")
    }

    pub fn parse(version: &str) -> Result<Self, semver::Error> {
        Version::parse(version).map(Self::from)
    }
}

impl From<Version> for ContractVersion {
    fn from(semver: Version) -> Self {
        Self { semver }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParserInfo {
    pub name: String,
    pub version: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<ParserBuildInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParserBuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}
