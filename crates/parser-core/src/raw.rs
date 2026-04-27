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
