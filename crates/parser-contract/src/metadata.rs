use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::presence::FieldPresence;

/// Normalized replay-level metadata from observed OCAP top-level fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayMetadata {
    /// Observed mission name.
    pub mission_name: FieldPresence<String>,
    /// Observed world name.
    pub world_name: FieldPresence<String>,
    /// Observed mission author.
    pub mission_author: FieldPresence<String>,
    /// Observed player count timeline or values.
    pub players_count: FieldPresence<Vec<u32>>,
    /// Observed capture delay.
    pub capture_delay: FieldPresence<f64>,
    /// Observed final frame.
    pub end_frame: FieldPresence<u64>,
    /// Derived or observed time bounds.
    pub time_bounds: FieldPresence<ReplayTimeBounds>,
    /// Derived or observed frame bounds.
    pub frame_bounds: FieldPresence<FrameBounds>,
}

/// Replay time boundaries in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayTimeBounds {
    /// Start time in seconds.
    pub start_seconds: Option<f64>,
    /// End time in seconds.
    pub end_seconds: Option<f64>,
}

/// Replay frame boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FrameBounds {
    /// First known frame.
    pub start_frame: u64,
    /// Last known frame.
    pub end_frame: u64,
}
