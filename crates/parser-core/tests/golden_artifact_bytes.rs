//! Fast in-process byte-exact golden test.
//!
//! Parses the committed seed fixture through the public `parser_core::parse_replay`
//! API, serializes it exactly as the worker does (`serde_json::to_vec(&artifact)` plus
//! a trailing `b'\n'` — see `parser-worker/src/processor.rs:149-150`), and asserts the
//! bytes equal the committed baseline byte-for-byte.
//!
//! This is the fast layer of the golden container e2e oracle: it shares the SAME
//! `valid-minimal.expected.json` baseline and the SAME pinned identity constants
//! (`tests/common/golden_identity.rs`) as `parser-worker/tests/golden_container_e2e.rs`,
//! so parser drift during the v1.1 behavior-preserving refactor fails here immediately
//! without Docker or containers.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use std::path::Path;

use parser_contract::{
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, public_parse_replay};
use serde_json::json;

// Shared, single-source-of-truth identity constants. Included (not imported) because
// the worker test crate textually includes the same file via a relative path.
include!("common/golden_identity.rs");

const SEED_FIXTURE: &str = "tests/fixtures/valid-minimal.ocap.json";
const EXPECTED_BASELINE: &[u8] =
    include_bytes!("fixtures/golden/expected/valid-minimal.expected.json");

fn golden_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some(GOLDEN_REPLAY_ID.to_owned()),
        source_file: GOLDEN_OBJECT_KEY.to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(GOLDEN_SOURCE_CHECKSUM_HEX)
                .expect("pinned golden checksum should be valid SHA-256"),
            source: None,
        },
    }
}

fn golden_parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": GOLDEN_PARSER_NAME,
        "version": GOLDEN_PARSER_VERSION,
    }))
    .expect("golden parser info should be valid")
}

#[test]
fn golden_artifact_bytes_should_match_committed_baseline_byte_for_byte() {
    // Arrange
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SEED_FIXTURE);
    let bytes = std::fs::read(&fixture_path).expect("seed fixture should be readable");

    // Act: reproduce the worker's exact artifact serialization. The worker uses
    // public_parse_replay (processor.rs:136) — the public minimal artifact with
    // debug-only per-field source provenance stripped — then serde_json::to_vec +
    // trailing b'\n' (processor.rs:149-150). parse_replay would retain provenance and
    // NOT match the S3 bytes, so this MUST use public_parse_replay.
    let artifact = public_parse_replay(ParserInput {
        bytes: &bytes,
        source: golden_source(),
        parser: golden_parser_info(),
        options: ParserOptions::default(),
    });
    let mut produced = serde_json::to_vec(&artifact).expect("artifact should serialize");
    produced.push(b'\n');

    // Assert: byte-for-byte, including the trailing newline.
    assert_eq!(
        produced, EXPECTED_BASELINE,
        "parse_replay output bytes drifted from the committed golden baseline; \
         if this is an intentional contract change, regenerate \
         tests/fixtures/golden/expected/valid-minimal.expected.json"
    );
}
