use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    events::EventActorRef,
    identity::EntitySide,
    presence::{Confidence, FieldPresence, UnknownReason},
    source_ref::{RuleId, SourceRefs},
};

/// Replay-level side facts for commander and outcome integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySideFacts {
    /// Commander facts observed or inferred for replay sides.
    pub commanders: Vec<CommanderSideFact>,
    /// Replay outcome fact.
    pub outcome: OutcomeFact,
}

impl Default for ReplaySideFacts {
    fn default() -> Self {
        Self { commanders: Vec::new(), outcome: OutcomeFact::unknown() }
    }
}

impl ReplaySideFacts {
    /// Returns true when no replay-side evidence is present.
    #[must_use]
    pub fn is_default(value: &Self) -> bool {
        value == &Self::default()
    }
}

/// Commander fact evidence kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommanderFactKind {
    /// Commander was directly observed in source data.
    Observed,
    /// Commander is a conservative candidate from parser heuristics.
    Candidate,
}

/// Commander fact for one replay side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CommanderSideFact {
    /// Observed side identifier.
    pub side: FieldPresence<EntitySide>,
    /// Observed side name or label.
    pub side_name: FieldPresence<String>,
    /// Commander actor reference.
    pub commander: FieldPresence<EventActorRef>,
    /// Fact kind.
    pub kind: CommanderFactKind,
    /// Confidence for inferred or candidate facts.
    pub confidence: Option<Confidence>,
    /// Rule that produced this fact.
    pub rule_id: RuleId,
    /// Source references backing the fact.
    pub source_refs: SourceRefs,
}

/// Outcome fact status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeStatus {
    /// Outcome is directly known.
    Known,
    /// Outcome is absent or unavailable.
    Unknown,
    /// Outcome was inferred from source evidence.
    Inferred,
}

/// Replay outcome fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OutcomeFact {
    /// Outcome status.
    pub status: OutcomeStatus,
    /// Winner side when known or inferred.
    pub winner_side: FieldPresence<EntitySide>,
    /// Raw source outcome label when present.
    pub source_label: FieldPresence<String>,
    /// Confidence for inferred outcomes.
    pub confidence: Option<Confidence>,
    /// Rule that produced this fact.
    pub rule_id: RuleId,
    /// Source references backing known or inferred outcomes.
    pub source_refs: Option<SourceRefs>,
}

impl OutcomeFact {
    /// Creates the default explicit unknown outcome fact.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            status: OutcomeStatus::Unknown,
            winner_side: FieldPresence::Unknown {
                reason: UnknownReason::MissingWinner,
                source: None,
            },
            source_label: FieldPresence::Unknown {
                reason: UnknownReason::MissingWinner,
                source: None,
            },
            confidence: None,
            rule_id: RuleId("side_facts.outcome.unknown".to_string()),
            source_refs: None,
        }
    }
}
