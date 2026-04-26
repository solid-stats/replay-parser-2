use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};

/// Current parser artifact contract version.
pub const CURRENT_CONTRACT_VERSION: &str = "1.0.0";

/// Semantic parser artifact contract version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct ContractVersion {
    /// Contract semantic version.
    pub semver: Version,
}

impl ContractVersion {
    /// Returns the current parser artifact contract version.
    #[must_use]
    pub const fn current() -> Self {
        Self { semver: Version::new(1, 0, 0) }
    }

    /// Parses a parser artifact contract version.
    ///
    /// # Errors
    ///
    /// Returns [`semver::Error`] when the version is not valid semantic version syntax.
    pub fn parse(version: &str) -> Result<Self, semver::Error> {
        Version::parse(version).map(Self::from)
    }
}

impl From<Version> for ContractVersion {
    fn from(semver: Version) -> Self {
        Self { semver }
    }
}

/// Parser binary metadata embedded in each artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParserInfo {
    /// Parser binary name.
    pub name: String,
    /// Parser binary version.
    pub version: Version,
    /// Optional build metadata for non-deterministic adapter contexts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<ParserBuildInfo>,
}

/// Optional parser build metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParserBuildInfo {
    /// Git commit used to build the parser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    /// Rust compilation target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Cargo profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}
