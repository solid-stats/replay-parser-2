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

// ---------------------------------------------------------------------------------------
// Real-corpus golden fixtures (committed-gzipped-real-corpus)
// ---------------------------------------------------------------------------------------
//
// Byte-exact REAL OCAP replays pulled from Timeweb S3 (bucket `sg-replays`, key
// `raw/sha256/<sha>.ocap`), gzipped at rest under `tests/fixtures/golden/real/<sha>.ocap.gz`.
// Each entry pins the STABLE source identity that the worker embeds into the artifact, so
// the committed baseline is reproducible byte-for-byte by BOTH golden consumers:
//   - `crates/parser-core/tests/golden_artifact_bytes.rs` (fast, no Docker), and
//   - `crates/parser-worker/tests/golden_container_e2e.rs` (testcontainers e2e).
//
// DETERMINISM TRAP: the artifact embeds `source.source_file` + `source.checksum`. The
// worker sets `source_file` to the S3 OBJECT KEY (deterministic), so the baseline must use
// the SAME stable object key — NEVER a local `/tmp` path. We mirror the real S3 layout:
//   replay_id  = the fixture sha256 (deterministic)
//   object_key = "raw/sha256/<sha>.ocap" (the real Timeweb S3 key — deterministic)
//   checksum   = the fixture's real sha256 (stable; equals the file name)
//
// To refresh a baseline after an intentional contract change, run the regeneration test:
//   `cargo test -p parser-core --test golden_artifact_bytes -- --ignored regenerate`

/// One real-corpus golden fixture: a gzipped raw OCAP input paired with its byte-exact
/// worker-output baseline, plus the pinned identity the worker embeds into the artifact.
#[derive(Clone, Copy, Debug)]
pub struct GoldenRealFixture {
    /// Short stable label used in assertion messages.
    pub label: &'static str,
    /// File stem under `tests/fixtures/golden/real/` (the fixture sha256), without `.ocap.gz`.
    pub sha256: &'static str,
    /// Baseline file name under `tests/fixtures/golden/expected/`.
    pub baseline_file: &'static str,
    /// Expected `ParseStatus` ("success" or "partial").
    pub expected_status: &'static str,
}

impl GoldenRealFixture {
    /// Deterministic replay id embedded into the artifact (the fixture sha256).
    #[must_use]
    pub const fn replay_id(&self) -> &'static str {
        self.sha256
    }
}

/// The committed real-corpus spread: small/mid/large success + one partial.
///
/// All four are byte-exact REAL replays, sha256-verified, pulled from Timeweb S3
/// `sg-replays raw/sha256/<sha>.ocap` on 2026-06-18.
pub const GOLDEN_REAL_FIXTURES: [GoldenRealFixture; 4] = [
    GoldenRealFixture {
        label: "real-small-success",
        sha256: "00118b2386dc8ba66ada1a8e956cbe9b983c0af280beef0ffbeeb86435c57969",
        baseline_file: "real-small-success.expected.json",
        expected_status: "success",
    },
    GoldenRealFixture {
        label: "real-mid-success",
        sha256: "0006b10dfbbb6d6fcb387fb66cb62d40740e4aa93ba9a2ef81ceb7f47af1e7c3",
        baseline_file: "real-mid-success.expected.json",
        expected_status: "success",
    },
    GoldenRealFixture {
        label: "real-large-success",
        sha256: "0053d62b12624e974c8a19a54816a08661dda35d12ed0462ca5f616bbbc9a3e6",
        baseline_file: "real-large-success.expected.json",
        expected_status: "success",
    },
    GoldenRealFixture {
        label: "real-partial-schema-drift",
        sha256: "00085e0394ad4125a822e703ea41228bf7b48495d8e0877a235a6d7b88014b9f",
        baseline_file: "real-partial-schema-drift.expected.json",
        expected_status: "partial",
    },
];

/// The deterministic S3 object key a real-corpus fixture is seeded under.
///
/// Mirrors the real Timeweb layout `raw/sha256/<sha>.ocap`, used as
/// `ReplaySource::source_file` so the committed baseline never embeds a local path.
/// `format!` is fine here — this file is only ever `include!`d into test crates.
#[must_use]
pub fn golden_real_object_key(sha256: &str) -> String {
    format!("raw/sha256/{sha256}.ocap")
}
