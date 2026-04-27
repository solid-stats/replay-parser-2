//! Pure parser core for deterministic `SolidGames` replay artifacts.
//!
//! This crate accepts caller-provided replay bytes and source metadata, then returns
//! versioned `parser-contract` artifacts. Runtime adapters own file access, queues,
//! object storage, databases, and non-deterministic timestamps.

pub mod artifact;
pub mod diagnostics;
pub mod entities;
pub mod input;
pub mod metadata;
pub mod raw;

pub use input::{ParserInput, ParserOptions};

/// Parses replay bytes into a versioned parser artifact.
#[must_use]
pub fn parse_replay(input: ParserInput<'_>) -> parser_contract::artifact::ParseArtifact {
    artifact::parse_replay(input)
}
