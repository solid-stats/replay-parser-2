// coverage-exclusion: reviewed Phase 05 validation/deserialization defensive branches are allowlisted by exact source line.
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::source_ref::{RuleId, SourceRef};

/// Explicit state wrapper for fields that may be present, null, unknown, inferred, or irrelevant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FieldPresence<T> {
    /// Field was present in the source or adapter input.
    Present {
        /// Present value.
        value: T,
        /// Source reference for the value, when available.
        source: Option<SourceRef>,
    },
    /// Field was explicitly null in the source or domain event.
    ExplicitNull {
        /// Reason the field is null.
        reason: NullReason,
        /// Source reference for the null state, when available.
        source: Option<SourceRef>,
    },
    /// Field value is unknown.
    Unknown {
        /// Reason the field is unknown.
        reason: UnknownReason,
        /// Source reference for the unknown state, when available.
        source: Option<SourceRef>,
    },
    /// Field value was inferred rather than directly observed.
    Inferred {
        /// Inferred value.
        value: T,
        /// Human-readable inference reason.
        reason: String,
        /// Optional confidence score for the inference.
        confidence: Option<Confidence>,
        /// Source reference for the inference, when available.
        source: Option<SourceRef>,
        /// Rule that produced the inference.
        rule_id: RuleId,
    },
    /// Field does not apply to this replay, entity, event, or projection.
    NotApplicable {
        /// Reason the field is not applicable.
        reason: String,
    },
}

/// Reason for an explicit null field state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NullReason {
    /// Killer is explicitly null.
    NullKiller,
    /// Source field value was null.
    SourceNull,
    /// Source field was empty and normalized to null.
    EmptyValue,
}

/// Reason for an unknown field state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownReason {
    /// `SteamID` is absent from the source.
    MissingSteamId,
    /// Winner is absent from the source.
    MissingWinner,
    /// Commander data is absent from the source.
    AbsentCommander,
    /// Source field is absent.
    SourceFieldAbsent,
    /// Source schema drift prevents a reliable value.
    SchemaDrift,
    /// Checksum could not be calculated or supplied.
    ChecksumUnavailable,
}

/// Confidence score for inferred field values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Confidence(#[schemars(range(min = 0.0, max = 1.0))] f32);

impl Confidence {
    /// Creates a confidence score in the inclusive range `0.0..=1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfidenceError`] when the value is not finite or is outside the inclusive
    /// confidence range.
    pub fn new(value: f32) -> Result<Self, ConfidenceError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ConfidenceError);
        }

        Ok(Self(value))
    }

    /// Returns the inner confidence value.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f32::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

/// Error returned when a confidence score is outside the accepted range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("confidence must be finite and between 0.0 and 1.0")]
pub struct ConfidenceError;
