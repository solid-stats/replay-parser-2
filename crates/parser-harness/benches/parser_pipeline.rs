//! Parser-stage benchmark entrypoints for Phase 05.

#![allow(
    missing_docs,
    unused_results,
    reason = "criterion macros and builder-style benchmark registration emit public harness items"
)]
#![allow(
    clippy::expect_used,
    reason = "benchmarks use expect messages to fail loudly on invalid fixtures"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use parser_contract::{
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::{Value, json};

const AGGREGATE_FIXTURE: &[u8] =
    include_bytes!("../../parser-core/tests/fixtures/aggregate-combat.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("benchmark parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("phase-05-benchmark".to_owned()),
        source_file: "fixtures/aggregate-combat.ocap.json".to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "6666666666666666666666666666666666666666666666666666666666666666",
            )
            .expect("benchmark checksum should be valid"),
            source: None,
        },
    }
}

fn parser_input(bytes: &[u8]) -> ParserInput<'_> {
    ParserInput {
        bytes,
        source: replay_source(),
        parser: parser_info(),
        options: ParserOptions::default(),
    }
}

fn parse_only(criterion: &mut Criterion) {
    criterion.bench_function("parse_only_json_decode", |bencher| {
        bencher.iter(|| {
            serde_json::from_slice::<Value>(black_box(AGGREGATE_FIXTURE))
                .expect("benchmark fixture should decode as JSON")
        });
    });
}

fn aggregate_only(criterion: &mut Criterion) {
    let artifact = parse_replay(parser_input(AGGREGATE_FIXTURE));

    criterion.bench_function("aggregate_only_public_projection_access", |bencher| {
        bencher.iter(|| {
            let projections = black_box(&artifact.aggregates.projections);
            projections
                .get("legacy.player_game_results")
                .expect("aggregate fixture should include player projection")
                .clone()
        });
    });
}

fn end_to_end(criterion: &mut Criterion) {
    criterion.bench_function("end_to_end_parse_replay", |bencher| {
        bencher.iter(|| parse_replay(parser_input(black_box(AGGREGATE_FIXTURE))));
    });
}

criterion_group!(parser_pipeline, parse_only, aggregate_only, end_to_end);
criterion_main!(parser_pipeline);
