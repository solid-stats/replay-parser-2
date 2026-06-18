//! Fast in-process byte-exact golden test.
//!
//! Parses each committed golden fixture through the public `parser_core::public_parse_replay`
//! API, serializes it exactly as the worker does (`serde_json::to_vec(&artifact)` plus a
//! trailing `b'\n'` — see `parser-worker/src/processor.rs:149-150`), and asserts the bytes
//! equal the committed baseline byte-for-byte.
//!
//! This is the fast layer of the golden container e2e oracle: it shares the SAME baselines
//! and the SAME pinned identity constants (`tests/common/golden_identity.rs`) as
//! `parser-worker/tests/golden_container_e2e.rs`, so parser drift during the v1.1
//! behavior-preserving refactor fails here immediately without Docker or containers.
//!
//! Two fixture tiers are baselined and consumed by BOTH layers:
//!   * `valid-minimal` — the tiny hand-focused seed (identity from the `GOLDEN_*` consts).
//!   * the real-corpus spread (`GOLDEN_REAL_FIXTURES`) — byte-exact REAL OCAP replays
//!     pulled from Timeweb S3, gzipped at rest and gunzipped at runtime.
//!
//! Regenerate every baseline after an intentional contract change with:
//!   `cargo test -p parser-core --test golden_artifact_bytes -- --ignored regenerate`

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{io::Read, path::Path, path::PathBuf};

use flate2::read::GzDecoder;
use parser_contract::{
    artifact::ParseArtifact,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, public_parse_replay};
use serde_json::json;

// Shared, single-source-of-truth identity constants + the real-corpus fixture table.
// Included (not imported) because the worker test crate textually includes the same file.
include!("common/golden_identity.rs");

const VALID_MINIMAL_FIXTURE: &str = "tests/fixtures/valid-minimal.ocap.json";
const VALID_MINIMAL_BASELINE: &str = "tests/fixtures/golden/expected/valid-minimal.expected.json";
const REAL_DIR: &str = "tests/fixtures/golden/real";
const EXPECTED_DIR: &str = "tests/fixtures/golden/expected";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn parser_info(version: &str) -> ParserInfo {
    serde_json::from_value(json!({ "name": GOLDEN_PARSER_NAME, "version": version }))
        .expect("golden parser info should be valid")
}

fn replay_source(replay_id: &str, object_key: &str, checksum_hex: &str) -> ReplaySource {
    ReplaySource {
        replay_id: Some(replay_id.to_owned()),
        source_file: object_key.to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(checksum_hex)
                .expect("pinned golden checksum should be valid SHA-256"),
            source: None,
        },
    }
}

/// Reproduces the EXACT bytes the worker writes to S3: `public_parse_replay` (the public
/// minimal artifact, debug-only provenance stripped) → `serde_json::to_vec` + trailing
/// `b'\n'` (processor.rs:136-150). `parse_replay` would retain provenance and NOT match the
/// S3 bytes, so this MUST use `public_parse_replay`.
fn produce_artifact(bytes: &[u8], source: ReplaySource, version: &str) -> ParseArtifact {
    public_parse_replay(ParserInput {
        bytes,
        source,
        parser: parser_info(version),
        options: ParserOptions::default(),
    })
}

fn artifact_bytes(artifact: &ParseArtifact) -> Vec<u8> {
    let mut produced = serde_json::to_vec(artifact).expect("artifact should serialize");
    produced.push(b'\n');
    produced
}

/// Gunzips a committed real-corpus fixture at runtime (gzip-at-rest).
fn read_real_fixture(sha256: &str) -> Vec<u8> {
    let path = crate_root().join(REAL_DIR).join(format!("{sha256}.ocap.gz"));
    let gz = std::fs::read(&path).unwrap_or_else(|error| {
        panic!("real fixture {} should be readable: {error}", path.display())
    });
    let mut decoder = GzDecoder::new(gz.as_slice());
    let mut raw = Vec::new();
    let _decoded = decoder.read_to_end(&mut raw).expect("real fixture should gunzip");
    raw
}

#[test]
fn golden_artifact_bytes_should_match_valid_minimal_baseline_byte_for_byte() {
    // Arrange
    let bytes = std::fs::read(crate_root().join(VALID_MINIMAL_FIXTURE))
        .expect("valid-minimal seed fixture should be readable");
    let expected = std::fs::read(crate_root().join(VALID_MINIMAL_BASELINE))
        .expect("valid-minimal baseline should be readable");

    // Act
    let source = replay_source(GOLDEN_REPLAY_ID, GOLDEN_OBJECT_KEY, GOLDEN_SOURCE_CHECKSUM_HEX);
    let produced = artifact_bytes(&produce_artifact(&bytes, source, GOLDEN_PARSER_VERSION));

    // Assert
    assert_eq!(
        produced, expected,
        "valid-minimal artifact bytes drifted from the committed golden baseline; if this is \
         an intentional contract change, run the `regenerate` test (see file header)"
    );
}

#[test]
fn golden_artifact_bytes_should_match_real_corpus_baselines_byte_for_byte() {
    for fixture in GOLDEN_REAL_FIXTURES {
        // Arrange
        let raw = read_real_fixture(fixture.sha256);
        let baseline = std::fs::read(crate_root().join(EXPECTED_DIR).join(fixture.baseline_file))
            .unwrap_or_else(|error| {
                panic!("baseline {} should be readable: {error}", fixture.baseline_file)
            });

        // Act
        let source = replay_source(
            fixture.replay_id(),
            &golden_real_object_key(fixture.sha256),
            fixture.sha256,
        );
        let artifact = produce_artifact(&raw, source, GOLDEN_PARSER_VERSION);

        // Assert: status matches the manifest claim, then byte-for-byte against the baseline.
        let status = serde_json::to_value(artifact.status)
            .expect("status should serialize")
            .as_str()
            .expect("status should be a string")
            .to_owned();
        assert_eq!(
            status, fixture.expected_status,
            "real fixture `{}` ({}) parsed to status `{status}`, expected `{}`",
            fixture.label, fixture.sha256, fixture.expected_status
        );
        assert_eq!(
            artifact_bytes(&artifact),
            baseline,
            "real fixture `{}` ({}) artifact bytes drifted from its committed baseline {}; if \
             this is an intentional contract change, run the `regenerate` test (see file header)",
            fixture.label,
            fixture.sha256,
            fixture.baseline_file
        );
    }
}

/// Regenerates EVERY committed baseline from the current parser output. Not run in the
/// normal suite (`#[ignore]`); run deliberately after an intentional contract change:
///   `cargo test -p parser-core --test golden_artifact_bytes -- --ignored regenerate`
#[test]
#[ignore = "writes committed baselines; run deliberately after an intentional contract change"]
fn regenerate_golden_baselines() {
    // valid-minimal
    let bytes = std::fs::read(crate_root().join(VALID_MINIMAL_FIXTURE))
        .expect("valid-minimal seed fixture should be readable");
    let source = replay_source(GOLDEN_REPLAY_ID, GOLDEN_OBJECT_KEY, GOLDEN_SOURCE_CHECKSUM_HEX);
    let produced = artifact_bytes(&produce_artifact(&bytes, source, GOLDEN_PARSER_VERSION));
    std::fs::write(crate_root().join(VALID_MINIMAL_BASELINE), &produced)
        .expect("valid-minimal baseline should write");

    // real corpus
    for fixture in GOLDEN_REAL_FIXTURES {
        let raw = read_real_fixture(fixture.sha256);
        let source = replay_source(
            fixture.replay_id(),
            &golden_real_object_key(fixture.sha256),
            fixture.sha256,
        );
        let produced = artifact_bytes(&produce_artifact(&raw, source, GOLDEN_PARSER_VERSION));
        let out = crate_root().join(EXPECTED_DIR).join(fixture.baseline_file);
        std::fs::write(&out, &produced)
            .unwrap_or_else(|error| panic!("baseline {} should write: {error}", out.display()));
        assert!(Path::new(&out).is_file());
    }
}
