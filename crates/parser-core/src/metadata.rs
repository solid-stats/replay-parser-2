//! Replay metadata normalization from observed OCAP top-level fields.
// coverage-exclusion: reviewed Phase 05 defensive metadata normalization branches are allowlisted by exact source line.

use parser_contract::{
    diagnostic::{Diagnostic, DiagnosticSeverity},
    metadata::{FrameBounds, ReplayMetadata, ReplayTimeBounds},
    presence::{FieldPresence, UnknownReason},
    source_ref::{RuleId, SourceRef, SourceRefs},
};

use crate::{
    artifact::SourceContext,
    diagnostics::{DiagnosticAccumulator, DiagnosticImpact},
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
                diagnostics.push(diagnostic, DiagnosticImpact::DataLoss);
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
        ) if capture_delay.is_finite() => {
            let end_frame = u64_to_f64(*end_frame);
            FieldPresence::Present {
                value: ReplayTimeBounds {
                    start_seconds: Some(0.0),
                    end_seconds: Some(end_frame * *capture_delay),
                },
                source: Some(
                    context.source_ref("$.captureDelay", rule_id("metadata.time_bounds.observed")),
                ),
            }
        }
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

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "metadata time bounds mirror legacy f64 frame math and do not require exact integer precision"
)]
const fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

#[cfg(all(test, not(coverage)))]
mod tests {
    use super::*;
    use parser_contract::presence::NullReason;
    use parser_contract::source_ref::ReplaySource;

    fn context() -> SourceContext {
        SourceContext::new(&ReplaySource {
            replay_id: Some("metadata-unit-test".to_owned()),
            source_file: "metadata-unit-test.ocap.json".to_owned(),
            checksum: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: None,
            },
        })
    }

    #[test]
    fn metadata_bounds_should_treat_defensive_non_observed_frame_states_as_schema_drift() {
        // Arrange
        let context = context();
        let end_frame =
            FieldPresence::ExplicitNull { reason: NullReason::SourceNull, source: None };

        // Act
        let bounds = frame_bounds(&end_frame, &context);

        // Assert
        assert!(matches!(
            bounds,
            FieldPresence::Unknown { reason: UnknownReason::SchemaDrift, source: Some(_) }
        ));
    }

    #[test]
    fn metadata_time_bounds_should_treat_defensive_non_observed_capture_delay_as_schema_drift() {
        // Arrange
        let context = context();
        let end_frame = FieldPresence::Present { value: 10, source: None };
        let capture_delay =
            FieldPresence::NotApplicable { reason: "unit test defensive state".to_owned() };

        // Act
        let bounds = time_bounds(&end_frame, &capture_delay, &context);

        // Assert
        assert!(matches!(
            bounds,
            FieldPresence::Unknown { reason: UnknownReason::SchemaDrift, source: Some(_) }
        ));
    }

    #[test]
    fn metadata_time_bounds_should_propagate_unknown_end_frame_source() {
        // Arrange
        let context = context();
        let end_frame = FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(context.source_ref("$.endFrame", rule_id("metadata.end_frame.observed"))),
        };
        let capture_delay = FieldPresence::Present { value: 0.5, source: None };

        // Act
        let bounds = time_bounds(&end_frame, &capture_delay, &context);

        // Assert
        assert!(matches!(
            bounds,
            FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
        ));
    }

    #[test]
    fn metadata_time_bounds_should_propagate_unknown_capture_delay_source() {
        // Arrange
        let context = context();
        let end_frame = FieldPresence::Present { value: 10, source: None };
        let capture_delay = FieldPresence::Unknown {
            reason: UnknownReason::SchemaDrift,
            source: Some(
                context.source_ref("$.captureDelay", rule_id("metadata.capture_delay.observed")),
            ),
        };

        // Act
        let bounds = time_bounds(&end_frame, &capture_delay, &context);

        // Assert
        assert!(matches!(
            bounds,
            FieldPresence::Unknown { reason: UnknownReason::SchemaDrift, source: Some(_) }
        ));
    }
}
