use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySource {
    pub replay_id: Option<String>,
    pub source_file: String,
    pub checksum: SourceChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceChecksum {
    pub algorithm: String,
    pub value: String,
}
