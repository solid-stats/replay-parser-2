//! Tolerant accessors for selective OCAP replay root fields.
// coverage-exclusion: reviewed Phase 05 defensive raw accessor branches are allowlisted by exact source line.
//!
//! This module keeps source JSON field-name and shape quirks at the parser-core boundary so
//! normalization code can consume explicit present, absent, or drifted observations.

use serde_json::value::RawValue;

pub use crate::raw_compact::{
    ConnectedEventObservation, KilledEventKillInfo, KilledEventObservation, RawEntityObservation,
};
use crate::raw_compact::{
    RawOcapRoot, compact_connected_events, compact_entities, compact_killed_events,
    observed_raw_shape, parse_raw_bool_or_numeric, parse_raw_f64, parse_raw_i64, parse_raw_string,
    parse_raw_u32_vec, parse_raw_u64, raw_array_field,
};

/// Borrowed wrapper around an OCAP replay root object.
#[derive(Debug, Clone, Copy)]
pub struct RawReplay<'a> {
    root: &'a RawOcapRoot<'a>,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires RawReplay field helpers to use borrowed receiver signatures"
)]
impl<'a> RawReplay<'a> {
    /// Creates a raw replay wrapper from a selectively decoded JSON object root.
    #[must_use]
    pub const fn new(root: &'a RawOcapRoot<'a>) -> Self {
        Self { root }
    }

    /// Returns a raw top-level value by source key.
    #[must_use]
    pub fn value_at(&self, key: &str) -> Option<&'a RawValue> {
        self.root.raw_value_at(key)
    }

    /// Reads a string top-level field.
    #[must_use]
    pub fn string_field(&self, key: &str) -> RawField<String> {
        self.field(key, "string", parse_raw_string)
    }

    /// Reads an unsigned 64-bit integer top-level field.
    #[must_use]
    pub fn u64_field(&self, key: &str) -> RawField<u64> {
        self.field(key, "unsigned integer", parse_raw_u64)
    }

    /// Reads a floating-point number top-level field.
    #[must_use]
    pub fn f64_field(&self, key: &str) -> RawField<f64> {
        self.field(key, "number", parse_raw_f64)
    }

    /// Reads an array of unsigned 32-bit integer values from a top-level field.
    #[must_use]
    pub fn u32_vec_field(&self, key: &str) -> RawField<Vec<u32>> {
        self.field(key, "array<unsigned integer>", parse_raw_u32_vec)
    }

    /// Reads compact source entity rows from `$.entities`.
    #[must_use]
    pub fn entities_field(&self) -> RawField<Vec<RawEntityObservation<'a>>> {
        let json_path = json_path("entities");

        match raw_array_field(self.value_at("entities"), json_path) {
            RawField::Present { json_path, .. } => {
                RawField::Present { value: compact_entities(self.root), json_path }
            }
            RawField::Absent { json_path } => RawField::Absent { json_path },
            RawField::Drift { json_path, expected_shape, observed_shape } => {
                RawField::Drift { json_path, expected_shape, observed_shape }
            }
        }
    }

    fn field<T>(
        &self,
        key: &str,
        expected_shape: &'static str,
        parse: impl FnOnce(&'a RawValue) -> Option<T>,
    ) -> RawField<T> {
        let json_path = json_path(key);

        match self.value_at(key) {
            Some(value) => match parse(value) {
                Some(value) => RawField::Present { value, json_path },
                None => RawField::Drift {
                    json_path,
                    expected_shape,
                    observed_shape: observed_raw_shape(value),
                },
            },
            None => RawField::Absent { json_path },
        }
    }
}

/// Reads a numeric source entity identifier from an entry under `$.entities`.
#[must_use]
pub fn entity_id(entity: &RawEntityObservation<'_>, index: usize) -> RawField<i64> {
    entity_field(entity, index, "id", "integer", parse_raw_i64)
}

/// Reads a source entity type from an entry under `$.entities`.
#[must_use]
pub fn entity_type(entity: &RawEntityObservation<'_>, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "type")
}

/// Reads a source entity name from an entry under `$.entities`.
#[must_use]
pub fn entity_name(entity: &RawEntityObservation<'_>, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "name")
}

/// Reads a source entity class, falling back from `class` to `_class`.
#[must_use]
pub fn entity_class(entity: &RawEntityObservation<'_>, index: usize) -> RawField<String> {
    match entity_string_field(entity, index, "class") {
        RawField::Present { value, json_path } => RawField::Present { value, json_path },
        RawField::Absent { json_path } => match entity_string_field(entity, index, "_class") {
            RawField::Present { value, json_path } => RawField::Present { value, json_path },
            RawField::Absent { .. } => RawField::Absent { json_path },
            drift @ RawField::Drift { .. } => drift,
        },
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            match entity_string_field(entity, index, "_class") {
                RawField::Present { value, json_path } => RawField::Present { value, json_path },
                RawField::Absent { .. } | RawField::Drift { .. } => {
                    RawField::Drift { json_path, expected_shape, observed_shape }
                }
            }
        }
    }
}

/// Reads a source entity side from an entry under `$.entities`.
#[must_use]
pub fn entity_side(entity: &RawEntityObservation<'_>, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "side")
}

/// Reads a source entity group from an entry under `$.entities`.
#[must_use]
pub fn entity_group(entity: &RawEntityObservation<'_>, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "group")
}

/// Reads a source entity description from an entry under `$.entities`.
#[must_use]
pub fn entity_description(entity: &RawEntityObservation<'_>, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "description")
}

/// Reads a source entity player flag from an entry under `$.entities`.
#[must_use]
pub fn entity_is_player(entity: &RawEntityObservation<'_>, index: usize) -> RawField<bool> {
    entity_field(entity, index, "isPlayer", "boolean or 0/1 number", parse_raw_bool_or_numeric)
}

/// Returns true when an entity row carries a `positions` source field.
#[must_use]
pub fn entity_has_positions(entity: &RawEntityObservation<'_>, _index: usize) -> bool {
    entity.entity.as_ref().is_some_and(crate::raw_compact::RawEntityCompact::has_positions)
}

/// Reads connected-player event tuples shaped as `[frame, "connected", name, entity_id]`.
#[must_use]
pub fn connected_events(raw: RawReplay<'_>) -> Vec<ConnectedEventObservation> {
    compact_connected_events(raw.root)
}

/// Reads killed event tuples shaped as `[frame, "killed", killed_id, kill_info, distance]`.
#[must_use]
pub fn killed_events(raw: RawReplay<'_>) -> Vec<KilledEventObservation> {
    compact_killed_events(raw.root)
}

/// Raw string evidence from a top-level replay field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawStringCandidate {
    /// Source top-level key that carried the string value.
    pub key: String,
    /// Observed string value.
    pub value: String,
    /// JSON path to the source field.
    pub json_path: String,
}

/// Reads present top-level string candidates for the supplied keys.
#[must_use]
pub fn string_candidates(raw: RawReplay<'_>, keys: &[&str]) -> Vec<RawStringCandidate> {
    keys.iter()
        .filter_map(|key| match raw.string_field(key) {
            RawField::Present { value, json_path } => {
                Some(RawStringCandidate { key: (*key).to_string(), value, json_path })
            }
            RawField::Absent { .. } | RawField::Drift { .. } => None,
        })
        .collect()
}

/// Tolerant top-level field observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawField<T> {
    /// The field was present and matched the expected source shape.
    Present {
        /// Parsed field value.
        value: T,
        /// JSON path to the source field.
        json_path: String,
    },
    /// The source field was absent from the root object.
    Absent {
        /// JSON path that was checked.
        json_path: String,
    },
    /// The field was present, but its source shape did not match the expected shape.
    Drift {
        /// JSON path to the drifted source field.
        json_path: String,
        /// Expected source shape.
        expected_shape: &'static str,
        /// Observed source shape.
        observed_shape: String,
    },
}

fn json_path(key: &str) -> String {
    format!("$.{key}")
}

fn entity_string_field(
    entity: &RawEntityObservation<'_>,
    index: usize,
    key: &str,
) -> RawField<String> {
    entity_field(entity, index, key, "string", parse_raw_string)
}

fn entity_field<T>(
    entity: &RawEntityObservation<'_>,
    index: usize,
    key: &str,
    expected_shape: &'static str,
    parse: impl FnOnce(&RawValue) -> Option<T>,
) -> RawField<T> {
    let json_path = entity_json_path(index, key);

    let Some(compact_entity) = &entity.entity else {
        return RawField::Drift {
            json_path: format!("$.entities[{index}]"),
            expected_shape: "object",
            observed_shape: entity.observed_shape.clone().unwrap_or_else(|| "unknown".to_string()),
        };
    };

    match compact_entity.raw_value_at(key) {
        Some(value) => match parse(value) {
            Some(value) => RawField::Present { value, json_path },
            None => RawField::Drift {
                json_path,
                expected_shape,
                observed_shape: observed_raw_shape(value),
            },
        },
        None => RawField::Absent { json_path },
    }
}

fn entity_json_path(index: usize, key: &str) -> String {
    format!("$.entities[{index}].{key}")
}

#[cfg(all(test, not(coverage)))]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use super::*;
    use crate::raw_compact::decode_compact_root;

    #[test]
    fn raw_helpers_should_preserve_defensive_shape_branches() {
        // Arrange
        let root = decode_compact_root(
            br#"{
                "entities": [
                    {"_class": false},
                    {"class": null, "_class": "fallback-class"},
                    {"class": null}
                ]
            }"#,
        )
        .expect("test root should decode");
        let RawField::Present { value: entities, .. } = RawReplay::new(&root).entities_field()
        else {
            panic!("test entities should be present");
        };

        // Act + Assert
        assert!(matches!(
            entity_class(&entities[0], 0),
            RawField::Drift { json_path, observed_shape, .. }
                if json_path == "$.entities[0]._class" && observed_shape == "boolean"
        ));
        assert!(matches!(
            entity_class(&entities[1], 1),
            RawField::Present { value, json_path }
                if value == "fallback-class" && json_path == "$.entities[1]._class"
        ));
        assert!(matches!(
            entity_class(&entities[2], 2),
            RawField::Drift { json_path, observed_shape, .. }
                if json_path == "$.entities[2].class" && observed_shape == "null"
        ));
    }

    #[test]
    fn raw_helpers_should_report_non_object_entity_rows() {
        // Arrange
        let root =
            decode_compact_root(br#"{"entities":[false]}"#).expect("test root should decode");
        let RawField::Present { value: entities, .. } = RawReplay::new(&root).entities_field()
        else {
            panic!("test entities should be present");
        };

        // Act
        let entity_id = entity_id(&entities[0], 0);

        // Assert
        assert!(matches!(
            entity_id,
            RawField::Drift { json_path, observed_shape, .. }
                if json_path == "$.entities[0]" && observed_shape == "boolean"
        ));
    }
}
