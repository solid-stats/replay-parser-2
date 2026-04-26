use parser_contract::{
    presence::{FieldPresence, NullReason, UnknownReason},
    source_ref::RuleId,
};
use serde_json::json;

#[test]
fn field_presence_missing_steam_id_should_serialize_unknown_reason() {
    let steam_id: FieldPresence<String> = FieldPresence::Unknown {
        reason: UnknownReason::MissingSteamId,
        source: None,
    };

    let serialized = serde_json::to_value(&steam_id).expect("presence should serialize");

    assert_eq!(
        serialized,
        json!({
            "state": "unknown",
            "reason": "missing_steam_id",
            "source": null
        })
    );
}

#[test]
fn field_presence_null_killer_should_serialize_explicit_null_reason() {
    let killer: FieldPresence<i64> = FieldPresence::ExplicitNull {
        reason: NullReason::NullKiller,
        source: None,
    };

    let serialized = serde_json::to_value(&killer).expect("presence should serialize");

    assert_eq!(
        serialized,
        json!({
            "state": "explicit_null",
            "reason": "null_killer",
            "source": null
        })
    );
}

#[test]
fn field_presence_inferred_value_should_serialize_confidence_and_rule_id() {
    let side = FieldPresence::Inferred {
        value: "west".to_string(),
        reason: "connected event side backfill".to_string(),
        confidence: Some(0.75),
        source: None,
        rule_id: RuleId::new("identity.side.inferred").expect("test rule ID should be non-empty"),
    };

    let serialized = serde_json::to_value(&side).expect("presence should serialize");

    assert_eq!(
        serialized,
        json!({
            "state": "inferred",
            "value": "west",
            "reason": "connected event side backfill",
            "confidence": 0.75,
            "source": null,
            "rule_id": "identity.side.inferred"
        })
    );
}
