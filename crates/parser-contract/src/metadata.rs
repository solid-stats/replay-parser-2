use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::presence::FieldPresence;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayMetadata {
    pub mission_name: FieldPresence<String>,
    pub world_name: FieldPresence<String>,
    pub mission_author: FieldPresence<String>,
    pub players_count: FieldPresence<Vec<u32>>,
    pub capture_delay: FieldPresence<f64>,
    pub end_frame: FieldPresence<u64>,
    pub time_bounds: FieldPresence<ReplayTimeBounds>,
    pub frame_bounds: FieldPresence<FrameBounds>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayTimeBounds {
    pub start_seconds: Option<f64>,
    pub end_seconds: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FrameBounds {
    pub start_frame: u64,
    pub end_frame: u64,
}
