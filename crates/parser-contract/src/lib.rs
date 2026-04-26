//! Versioned parser contract types for `SolidGames` replay parser artifacts.

#[cfg(test)]
use jsonschema as _;

/// Aggregate projection and contribution-reference contract types.
pub mod aggregates;
/// Parse artifact envelope contract types.
pub mod artifact;
/// Structured parser diagnostic contract types.
pub mod diagnostic;
/// Normalized event contract types.
pub mod events;
/// Structured parse failure contract types.
pub mod failure;
/// Observed entity and identity contract types.
pub mod identity;
/// Replay metadata contract types.
pub mod metadata;
/// Explicit present, null, unknown, inferred, and not-applicable field states.
pub mod presence;
/// JSON Schema generation for parser artifacts.
pub mod schema;
/// Source-reference and provenance contract types.
pub mod source_ref;
/// Parser and contract version contract types.
pub mod version;
