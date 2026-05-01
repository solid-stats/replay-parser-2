//! Pure parser core for deterministic `SolidGames` replay artifacts.
//!
//! This crate accepts caller-provided replay bytes and source metadata, then returns
//! versioned `parser-contract` artifacts. Runtime adapters own file access, queues,
//! object storage, databases, and non-deterministic timestamps.

pub mod aggregates;
pub mod artifact;
pub mod debug_artifact;
pub mod diagnostics;
pub mod entities;
pub mod events;
pub mod input;
pub mod legacy_player;
pub mod metadata;
pub mod raw;
pub mod raw_compact;
pub mod side_facts;

pub use debug_artifact::DebugParseArtifact;
pub use input::{ParserInput, ParserOptions};

/// Parses replay bytes into a versioned parser artifact.
#[must_use]
pub fn parse_replay(input: ParserInput<'_>) -> parser_contract::artifact::ParseArtifact {
    artifact::parse_replay(input)
}

/// Parses replay bytes into a full deterministic parser-side debug artifact.
#[must_use]
pub fn parse_replay_debug(input: ParserInput<'_>) -> DebugParseArtifact {
    debug_artifact::parse_replay_debug(input)
}
