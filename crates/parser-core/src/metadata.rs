//! Replay metadata normalization from observed OCAP top-level fields.

use parser_contract::{
    diagnostic::{Diagnostic, DiagnosticSeverity},
    metadata::{FrameBounds, ReplayMetadata, ReplayTimeBounds},
    presence::{FieldPresence, UnknownReason},
    source_ref::{RuleId, SourceRef, SourceRefs},
};

use crate::{
    artifact::SourceContext,
    diagnostics::DiagnosticAccumulator,
    raw::{RawField, RawReplay},
};

/// Normalizes replay metadata from tolerant raw OCAP top-level field observations.
#[must_use]
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires normalize_metadata to accept a borrowed RawReplay"
)]
pub fn normalize_metadata(
    raw: &RawReplay<'_>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> ReplayMetadata {
    let mission_name = normalize_field(
        raw.string_field("missionName"),
        context,
        diagnostics,
        "mission_name",
        "missionName",
    );
    let world_name = normalize_field(
        raw.string_field("worldName"),
        context,
        diagnostics,
        "world_name",
        "worldName",
    );
    let mission_author = normalize_field(
        raw.string_field("missionAuthor"),
        context,
        diagnostics,
        "mission_author",
        "missionAuthor",
    );
    let players_count = normalize_field(
        raw.u32_vec_field("playersCount"),
        context,
        diagnostics,
        "players_count",
        "playersCount",
    );
    let capture_delay = normalize_field(
        raw.f64_field("captureDelay"),
        context,
        diagnostics,
        "capture_delay",
        "captureDelay",
    );
    let end_frame =
        normalize_field(raw.u64_field("endFrame"), context, diagnostics, "end_frame", "endFrame");

    let frame_bounds = frame_bounds(&end_frame, context);
    let time_bounds = time_bounds(&end_frame, &capture_delay, context);

    ReplayMetadata {
        mission_name,
        world_name,
        mission_author,
        players_count,
        capture_delay,
        end_frame,
        time_bounds,
        frame_bounds,
    }
}

fn normalize_field<T>(
    raw_field: RawField<T>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
    contract_field: &'static str,
    source_key: &'static str,
) -> FieldPresence<T> {
    match raw_field {
        RawField::Present { value, json_path } => FieldPresence::Present {
            value,
            source: Some(metadata_source_ref(context, &json_path, contract_field)),
        },
        RawField::Absent { json_path } => FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(metadata_source_ref(context, &json_path, contract_field)),
        },
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            if let Some(diagnostic) = schema_drift_diagnostic(
                context,
                &json_path,
                expected_shape,
                &observed_shape,
                source_key,
            ) {
                diagnostics.push(diagnostic, true);
            }

            FieldPresence::Unknown {
                reason: UnknownReason::SchemaDrift,
                source: Some(metadata_source_ref(context, &json_path, contract_field)),
            }
        }
    }
}

fn frame_bounds(
    end_frame: &FieldPresence<u64>,
    context: &SourceContext,
) -> FieldPresence<FrameBounds> {
    match end_frame {
        FieldPresence::Present { value, source } => FieldPresence::Present {
            value: FrameBounds { start_frame: 0, end_frame: *value },
            source: source.as_ref().map(|_| {
                context.source_ref("$.endFrame", rule_id("metadata.frame_bounds.observed"))
            }),
        },
        FieldPresence::Unknown { reason, source } => FieldPresence::Unknown {
            reason: *reason,
            source: source.as_ref().map(|_| {
                context.source_ref("$.endFrame", rule_id("metadata.frame_bounds.observed"))
            }),
        },
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => FieldPresence::Unknown {
            reason: UnknownReason::SchemaDrift,
            source: Some(
                context.source_ref("$.endFrame", rule_id("metadata.frame_bounds.observed")),
            ),
        },
    }
}

fn time_bounds(
    end_frame: &FieldPresence<u64>,
    capture_delay: &FieldPresence<f64>,
    context: &SourceContext,
) -> FieldPresence<ReplayTimeBounds> {
    match (end_frame, capture_delay) {
        (
            FieldPresence::Present { value: end_frame, source: _ },
            FieldPresence::Present { value: capture_delay, source: _ },
        ) if capture_delay.is_finite() => u64_to_f64(*end_frame).map_or_else(
            || FieldPresence::Unknown {
                reason: UnknownReason::SchemaDrift,
                source: Some(
                    context.source_ref("$.endFrame", rule_id("metadata.time_bounds.observed")),
                ),
            },
            |end_frame| FieldPresence::Present {
                value: ReplayTimeBounds {
                    start_seconds: Some(0.0),
                    end_seconds: Some(end_frame * *capture_delay),
                },
                source: Some(
                    context.source_ref("$.captureDelay", rule_id("metadata.time_bounds.observed")),
                ),
            },
        ),
        (FieldPresence::Unknown { reason, source }, _)
        | (_, FieldPresence::Unknown { reason, source }) => FieldPresence::Unknown {
            reason: *reason,
            source: source.as_ref().map(|source| {
                context.source_ref(
                    source.json_path.as_deref().unwrap_or("$.captureDelay"),
                    rule_id("metadata.time_bounds.observed"),
                )
            }),
        },
        _ => FieldPresence::Unknown {
            reason: UnknownReason::SchemaDrift,
            source: Some(
                context.source_ref("$.captureDelay", rule_id("metadata.time_bounds.observed")),
            ),
        },
    }
}

fn metadata_source_ref(
    context: &SourceContext,
    json_path: &str,
    contract_field: &str,
) -> SourceRef {
    context.source_ref(json_path, rule_id(&format!("metadata.{contract_field}.observed")))
}

fn schema_drift_diagnostic(
    context: &SourceContext,
    json_path: &str,
    expected_shape: &str,
    observed_shape: &str,
    source_key: &str,
) -> Option<Diagnostic> {
    let source_ref = context.source_ref(json_path, rule_id("diagnostic.schema_drift.metadata"));
    let source_refs = SourceRefs::new(vec![source_ref]).ok()?;

    Some(Diagnostic {
        code: "schema.metadata_field".to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!("Metadata field {source_key} had unexpected source shape"),
        json_path: Some(json_path.to_string()),
        expected_shape: Some(expected_shape.to_string()),
        observed_shape: Some(observed_shape.to_string()),
        parser_action: "set_unknown".to_string(),
        source_refs,
    })
}

fn rule_id(value: &str) -> Option<RuleId> {
    RuleId::new(value).ok()
}

fn u64_to_f64(value: u64) -> Option<f64> {
    value.to_string().parse::<f64>().ok()
}
