//! Tolerant accessors for raw OCAP replay root fields.
//!
//! This module keeps source JSON field-name and shape quirks at the parser-core boundary so
//! normalization code can consume explicit present, absent, or drifted observations.

use serde_json::{Map, Value};

/// Borrowed wrapper around an OCAP replay root object.
#[derive(Debug, Clone, Copy)]
pub struct RawReplay<'a> {
    root: &'a Map<String, Value>,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires RawReplay field helpers to use borrowed receiver signatures"
)]
impl<'a> RawReplay<'a> {
    /// Creates a raw replay wrapper from a decoded JSON object root.
    #[must_use]
    pub const fn new(root: &'a Map<String, Value>) -> Self {
        Self { root }
    }

    /// Returns a raw top-level value by source key.
    #[must_use]
    pub fn value_at(&self, key: &str) -> Option<&'a Value> {
        self.root.get(key)
    }

    /// Reads a string top-level field.
    #[must_use]
    pub fn string_field(&self, key: &str) -> RawField<String> {
        self.field(key, "string", |value| value.as_str().map(ToOwned::to_owned))
    }

    /// Reads an unsigned 64-bit integer top-level field.
    #[must_use]
    pub fn u64_field(&self, key: &str) -> RawField<u64> {
        self.field(key, "unsigned integer", Value::as_u64)
    }

    /// Reads a floating-point number top-level field.
    #[must_use]
    pub fn f64_field(&self, key: &str) -> RawField<f64> {
        self.field(key, "number", Value::as_f64)
    }

    /// Reads an array of unsigned 32-bit integer values from a top-level field.
    #[must_use]
    pub fn u32_vec_field(&self, key: &str) -> RawField<Vec<u32>> {
        self.field(key, "array<unsigned integer>", |value| {
            let values = value.as_array()?;
            values.iter().map(|entry| u32::try_from(entry.as_u64()?).ok()).collect()
        })
    }

    /// Reads an array top-level field.
    #[must_use]
    pub fn array_field(&self, key: &str) -> RawField<&'a Vec<Value>> {
        self.field(key, "array", Value::as_array)
    }

    fn field<T>(
        &self,
        key: &str,
        expected_shape: &'static str,
        parse: impl FnOnce(&'a Value) -> Option<T>,
    ) -> RawField<T> {
        let json_path = json_path(key);

        match self.value_at(key) {
            Some(value) => match parse(value) {
                Some(value) => RawField::Present { value, json_path },
                None => RawField::Drift {
                    json_path,
                    expected_shape,
                    observed_shape: observed_shape(value),
                },
            },
            None => RawField::Absent { json_path },
        }
    }
}

/// Reads a numeric source entity identifier from an entry under `$.entities`.
#[must_use]
pub fn entity_id(entity: &Value, index: usize) -> RawField<i64> {
    entity_field(entity, index, "id", "integer", Value::as_i64)
}

/// Reads a source entity type from an entry under `$.entities`.
#[must_use]
pub fn entity_type(entity: &Value, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "type")
}

/// Reads a source entity name from an entry under `$.entities`.
#[must_use]
pub fn entity_name(entity: &Value, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "name")
}

/// Reads a source entity class, falling back from `class` to `_class`.
#[must_use]
pub fn entity_class(entity: &Value, index: usize) -> RawField<String> {
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
pub fn entity_side(entity: &Value, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "side")
}

/// Reads a source entity group from an entry under `$.entities`.
#[must_use]
pub fn entity_group(entity: &Value, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "group")
}

/// Reads a source entity description from an entry under `$.entities`.
#[must_use]
pub fn entity_description(entity: &Value, index: usize) -> RawField<String> {
    entity_string_field(entity, index, "description")
}

/// Reads a source entity player flag from an entry under `$.entities`.
#[must_use]
pub fn entity_is_player(entity: &Value, index: usize) -> RawField<bool> {
    entity_field(entity, index, "isPlayer", "boolean or 0/1 number", |value| {
        value.as_bool().or_else(|| {
            value.as_i64().and_then(|number| match number {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            })
        })
    })
}

/// Returns true when an entity row carries a `positions` source field.
#[must_use]
pub fn entity_has_positions(entity: &Value, _index: usize) -> bool {
    entity.as_object().is_some_and(|object| object.contains_key("positions"))
}

/// Raw connected-player event observation from `$.events`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedEventObservation {
    /// Original index in the source `events` array.
    pub event_index: usize,
    /// Source frame number from `event[0]`.
    pub frame: u64,
    /// Connected player name from `event[2]`.
    pub name: String,
    /// Source entity ID from `event[3]`.
    pub entity_id: i64,
    /// JSON path to the source event tuple.
    pub json_path: String,
}

/// Reads connected-player event tuples shaped as `[frame, "connected", name, entity_id]`.
#[must_use]
pub fn connected_events(raw: &RawReplay<'_>) -> Vec<ConnectedEventObservation> {
    let RawField::Present { value: events, json_path: _ } = raw.array_field("events") else {
        return Vec::new();
    };

    events
        .iter()
        .enumerate()
        .filter_map(|(event_index, event)| connected_event(event, event_index))
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

/// Returns a stable, coarse JSON shape name for diagnostics.
#[must_use]
pub fn observed_shape(value: &Value) -> String {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
    .to_string()
}

fn json_path(key: &str) -> String {
    format!("$.{key}")
}

fn entity_string_field(entity: &Value, index: usize, key: &str) -> RawField<String> {
    entity_field(entity, index, key, "string", |value| value.as_str().map(ToOwned::to_owned))
}

fn entity_field<T>(
    entity: &Value,
    index: usize,
    key: &str,
    expected_shape: &'static str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> RawField<T> {
    let json_path = entity_json_path(index, key);

    entity.as_object().map_or_else(
        || RawField::Drift {
            json_path: format!("$.entities[{index}]"),
            expected_shape: "object",
            observed_shape: observed_shape(entity),
        },
        |object| match object.get(key) {
            Some(value) => match parse(value) {
                Some(value) => RawField::Present { value, json_path },
                None => RawField::Drift {
                    json_path,
                    expected_shape,
                    observed_shape: observed_shape(value),
                },
            },
            None => RawField::Absent { json_path },
        },
    )
}

fn entity_json_path(index: usize, key: &str) -> String {
    format!("$.entities[{index}].{key}")
}

fn connected_event(event: &Value, event_index: usize) -> Option<ConnectedEventObservation> {
    let event = event.as_array()?;
    let frame = event.first()?.as_u64()?;
    let event_type = event.get(1)?.as_str()?;

    if event_type != "connected" {
        return None;
    }

    Some(ConnectedEventObservation {
        event_index,
        frame,
        name: event.get(2)?.as_str()?.to_string(),
        entity_id: event.get(3)?.as_i64()?,
        json_path: format!("$.events[{event_index}]"),
    })
}
