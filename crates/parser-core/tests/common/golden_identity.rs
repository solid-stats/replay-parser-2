// Shared byte-exact golden identity constants for the container e2e and the fast
// in-process golden suite.
//
// This file is the SINGLE source of truth for the pinned source identity that the
// worker embeds into the artifact (`replay_id`, `source_file`, `checksum`) plus the
// parser version. It is `include!`d verbatim by both:
//   - `crates/parser-core/tests/golden_artifact_bytes.rs` (fast, no Docker), and
//   - `crates/parser-worker/tests/golden_container_e2e.rs` (testcontainers e2e).
//
// Because the artifact bytes embed every value below, the committed baseline
// `crates/parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json`
// is only valid for THESE exact constants. Changing any of them (including a parser
// version bump) changes the bytes and requires regenerating the baseline.
//
// Do not add `use` statements here: the file is textually included into two crates
// whose imports differ. Reference all paths fully-qualified.

/// Pinned replay identity used by both the seed job message and the baseline source.
pub const GOLDEN_REPLAY_ID: &str = "replay-golden-minimal";

/// Pinned raw object key (becomes `ReplaySource::source_file`, embedded in the artifact).
pub const GOLDEN_OBJECT_KEY: &str = "raw/replay-golden-minimal.ocap.json";

/// Lowercase SHA-256 of `crates/parser-core/tests/fixtures/valid-minimal.ocap.json`.
///
/// Pinned as a constant so the baseline, the job message, and the assertions all
/// agree without recomputing. If the seed fixture bytes ever change, this constant
/// and the committed baseline must both be regenerated.
pub const GOLDEN_SOURCE_CHECKSUM_HEX: &str =
    "e41b8b54016a44259726474ee9b74cb5350ca23894e43732c37cde8d951d0eec";

/// Parser name embedded in the artifact (matches `parser_worker::runner::parser_info`).
pub const GOLDEN_PARSER_NAME: &str = "replay-parser-2";

/// Parser version embedded in the committed baseline artifact bytes.
///
/// PINNED, NOT derived from `env!("CARGO_PKG_VERSION")`: this file is textually
/// `include!`d into both `parser-core` and `parser-worker` test crates, and that macro
/// would resolve to the *including* crate's version. Today both crates are `0.1.0`, so
/// an `env!` happened to agree — but the moment the two crate versions diverge, exactly
/// one golden layer would fail with no behavior change, reading as spurious "parser
/// drift". A single pinned constant makes both consumers agree on the SAME value the
/// baseline actually carries. If this value changes, the committed baseline
/// `valid-minimal.expected.json` must be regenerated to match.
pub const GOLDEN_PARSER_VERSION: &str = "0.1.0";
