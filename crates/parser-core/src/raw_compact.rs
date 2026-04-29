//! Borrowed selective OCAP root extraction.
//!
//! This module deserializes the top-level replay object while borrowing heavy source
//! sections as raw JSON. Downstream accessors then deserialize only the fields and
//! event tuple slots required for v1 parser facts.

use serde::{
    Deserialize, Deserializer,
    de::{IgnoredAny, MapAccess, SeqAccess, Visitor},
};
use serde_json::{error::Category, value::RawValue};

use crate::raw::RawField;

/// Borrowed OCAP replay root with compact-first access to top-level facts.
#[derive(Debug)]
pub struct RawOcapRoot<'a> {
    mission_name: Option<&'a RawValue>,
    world_name: Option<&'a RawValue>,
    mission_author: Option<&'a RawValue>,
    players_count: Option<&'a RawValue>,
    capture_delay: Option<&'a RawValue>,
    end_frame: Option<&'a RawValue>,
    editor_markers: Option<&'a RawValue>,
    markers: Option<&'a RawValue>,
    entities: Option<&'a RawValue>,
    events: Option<&'a RawValue>,
    winner: Option<&'a RawValue>,
    winning_side: Option<&'a RawValue>,
    outcome: Option<&'a RawValue>,
}

impl<'a> RawOcapRoot<'a> {
    /// Returns a borrowed top-level raw value by source key.
    #[must_use]
    pub const fn raw_value_at(&self, key: &str) -> Option<&'a RawValue> {
        match key.as_bytes() {
            b"missionName" => self.mission_name,
            b"worldName" => self.world_name,
            b"missionAuthor" => self.mission_author,
            b"playersCount" => self.players_count,
            b"captureDelay" => self.capture_delay,
            b"endFrame" => self.end_frame,
            b"EditorMarkers" => self.editor_markers,
            b"Markers" => self.markers,
            b"entities" => self.entities,
            b"events" => self.events,
            b"winner" => self.winner,
            b"winningSide" => self.winning_side,
            b"outcome" => self.outcome,
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for RawOcapRoot<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RawOcapRootVisitor)
    }
}

struct RawOcapRootVisitor;

impl<'de> Visitor<'de> for RawOcapRootVisitor {
    type Value = RawOcapRoot<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an OCAP replay object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut root = RawOcapRoot {
            mission_name: None,
            world_name: None,
            mission_author: None,
            players_count: None,
            capture_delay: None,
            end_frame: None,
            editor_markers: None,
            markers: None,
            entities: None,
            events: None,
            winner: None,
            winning_side: None,
            outcome: None,
        };

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "missionName" => root.mission_name = Some(map.next_value()?),
                "worldName" => root.world_name = Some(map.next_value()?),
                "missionAuthor" => root.mission_author = Some(map.next_value()?),
                "playersCount" => root.players_count = Some(map.next_value()?),
                "captureDelay" => root.capture_delay = Some(map.next_value()?),
                "endFrame" => root.end_frame = Some(map.next_value()?),
                "EditorMarkers" => root.editor_markers = Some(map.next_value()?),
                "Markers" => root.markers = Some(map.next_value()?),
                "entities" => root.entities = Some(map.next_value()?),
                "events" => root.events = Some(map.next_value()?),
                "winner" => root.winner = Some(map.next_value()?),
                "winningSide" => root.winning_side = Some(map.next_value()?),
                "outcome" => root.outcome = Some(map.next_value()?),
                _ => {
                    let _ = map.next_value::<IgnoredAny>()?;
                }
            }
        }

        Ok(root)
    }
}

/// Decode failure category for the compact root boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CompactDecodeError {
    /// Replay bytes are not valid JSON syntax.
    #[error("replay JSON could not be decoded: {source_cause}")]
    JsonDecode {
        /// `serde_json` source error text.
        source_cause: String,
    },
    /// Replay bytes are valid JSON but the root is not an object.
    #[error("OCAP replay root must be a JSON object: {source_cause}")]
    RootNotObject {
        /// `serde_json` source error text.
        source_cause: String,
    },
}

/// Selectively decodes a replay root without constructing a full `serde_json::Value` DOM.
///
/// # Errors
///
/// Returns [`CompactDecodeError`] when the replay bytes are not valid JSON or the root is not a
/// JSON object.
pub fn decode_compact_root(bytes: &[u8]) -> Result<RawOcapRoot<'_>, CompactDecodeError> {
    serde_json::from_slice::<RawOcapRoot<'_>>(bytes).map_err(|error| match error.classify() {
        Category::Data => CompactDecodeError::RootNotObject {
            source_cause: "OCAP replay root must be a JSON object".to_string(),
        },
        Category::Io | Category::Syntax | Category::Eof => {
            CompactDecodeError::JsonDecode { source_cause: error.to_string() }
        }
    })
}

/// Compact borrowed entity row with only v1 parser-relevant source fields.
#[derive(Debug)]
pub struct RawEntityCompact<'a> {
    id: Option<&'a RawValue>,
    entity_type: Option<&'a RawValue>,
    name: Option<&'a RawValue>,
    class: Option<&'a RawValue>,
    alternate_class: Option<&'a RawValue>,
    side: Option<&'a RawValue>,
    group: Option<&'a RawValue>,
    description: Option<&'a RawValue>,
    is_player: Option<&'a RawValue>,
    steam_id_key: Option<&'a str>,
    steam_id: Option<&'a RawValue>,
    positions: Option<&'a RawValue>,
}

impl<'a> RawEntityCompact<'a> {
    /// Returns a borrowed raw entity field by source key.
    #[must_use]
    pub const fn raw_value_at(&self, key: &str) -> Option<&'a RawValue> {
        match key.as_bytes() {
            b"id" => self.id,
            b"type" => self.entity_type,
            b"name" => self.name,
            b"class" => self.class,
            b"_class" => self.alternate_class,
            b"side" => self.side,
            b"group" => self.group,
            b"description" => self.description,
            b"isPlayer" => self.is_player,
            b"steamID" | b"steamId" | b"steam_id" => self.steam_id,
            b"positions" => self.positions,
            _ => None,
        }
    }

    /// Returns the observed `SteamID` source key and value when any accepted alias is present.
    #[must_use]
    pub const fn raw_steam_id_field(&self) -> Option<(&'a str, &'a RawValue)> {
        match (self.steam_id_key, self.steam_id) {
            (Some(key), Some(value)) => Some((key, value)),
            _ => None,
        }
    }

    /// Returns true when the entity source row contains a `positions` field.
    #[must_use]
    pub const fn has_positions(&self) -> bool {
        self.positions.is_some()
    }
}

impl<'de> Deserialize<'de> for RawEntityCompact<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RawEntityCompactVisitor)
    }
}

struct RawEntityCompactVisitor;

impl<'de> Visitor<'de> for RawEntityCompactVisitor {
    type Value = RawEntityCompact<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an OCAP entity object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entity = RawEntityCompact {
            id: None,
            entity_type: None,
            name: None,
            class: None,
            alternate_class: None,
            side: None,
            group: None,
            description: None,
            is_player: None,
            steam_id_key: None,
            steam_id: None,
            positions: None,
        };

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "id" => entity.id = Some(map.next_value()?),
                "type" => entity.entity_type = Some(map.next_value()?),
                "name" => entity.name = Some(map.next_value()?),
                "class" => entity.class = Some(map.next_value()?),
                "_class" => entity.alternate_class = Some(map.next_value()?),
                "side" => entity.side = Some(map.next_value()?),
                "group" => entity.group = Some(map.next_value()?),
                "description" => entity.description = Some(map.next_value()?),
                "isPlayer" => entity.is_player = Some(map.next_value()?),
                "steamID" | "steamId" | "steam_id" => {
                    entity.steam_id_key = Some(key);
                    entity.steam_id = Some(map.next_value()?);
                }
                "positions" => entity.positions = Some(map.next_value()?),
                _ => {
                    let _ = map.next_value::<IgnoredAny>()?;
                }
            }
        }

        Ok(entity)
    }
}

/// Selective entity observation with stable source coordinates.
#[derive(Debug)]
pub struct RawEntityObservation<'a> {
    /// Source index in `$.entities`.
    pub index: usize,
    /// JSON path to the source entity row.
    pub json_path: String,
    /// Parsed compact object fields when the row is an object.
    pub entity: Option<RawEntityCompact<'a>>,
    /// Observed shape when the row is not an object.
    pub observed_shape: Option<String>,
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

/// Raw killed event observation from `$.events`.
#[derive(Debug, Clone, PartialEq)]
pub struct KilledEventObservation {
    /// Original index in the source `events` array.
    pub event_index: usize,
    /// Source frame number from `event[0]`, when numeric.
    pub frame: Option<u64>,
    /// Killed source entity ID from `event[2]`, when numeric.
    pub killed_entity_id: Option<i64>,
    /// Killer source entity and weapon evidence from `event[3]`.
    pub kill_info: KilledEventKillInfo,
    /// Source distance in meters from `event[4]`, when numeric.
    pub distance_meters: Option<f64>,
    /// JSON path to the source event tuple.
    pub json_path: String,
}

/// Raw killer evidence from a killed event tuple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KilledEventKillInfo {
    /// Source tuple explicitly records a null killer as `["null"]`.
    NullKiller,
    /// Source tuple records a numeric killer entity ID and optional weapon name.
    Killer {
        /// Killer source entity ID from `event[3][0]`.
        killer_entity_id: i64,
        /// Weapon name from `event[3][1]`, when it is a non-empty string.
        weapon: Option<String>,
    },
    /// Source tuple had an unrecognized `event[3]` shape.
    Malformed {
        /// Coarse observed JSON shape, or `absent` when the tuple entry is missing.
        observed_shape: String,
    },
}

/// Reads compact entity observations from the borrowed root.
#[must_use]
pub fn compact_entities<'a>(root: &'a RawOcapRoot<'a>) -> Vec<RawEntityObservation<'a>> {
    let Some(entities) = root.raw_value_at("entities") else {
        return Vec::new();
    };
    let Some(rows) = raw_array_items(entities) else {
        return Vec::new();
    };

    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            let entity = serde_json::from_str::<RawEntityCompact<'_>>(row.get()).ok();
            let observed_shape = entity.is_none().then(|| observed_raw_shape(row));
            RawEntityObservation {
                index,
                json_path: format!("$.entities[{index}]"),
                entity,
                observed_shape,
            }
        })
        .collect()
}

/// Reads connected-player event tuples shaped as `[frame, "connected", name, entity_id]`.
#[must_use]
pub fn compact_connected_events(root: &RawOcapRoot<'_>) -> Vec<ConnectedEventObservation> {
    event_rows(root)
        .into_iter()
        .filter_map(|(event_index, event)| connected_event(event, event_index))
        .collect()
}

/// Reads killed event tuples shaped as `[frame, "killed", killed_id, kill_info, distance]`.
#[must_use]
pub fn compact_killed_events(root: &RawOcapRoot<'_>) -> Vec<KilledEventObservation> {
    event_rows(root)
        .into_iter()
        .filter_map(|(event_index, event)| killed_event(event, event_index))
        .collect()
}

pub(crate) fn raw_array_field(
    value: Option<&RawValue>,
    json_path: String,
) -> RawField<Vec<&RawValue>> {
    match value {
        Some(value) => match raw_array_items(value) {
            Some(value) => RawField::Present { value, json_path },
            None => RawField::Drift {
                json_path,
                expected_shape: "array",
                observed_shape: observed_raw_shape(value),
            },
        },
        None => RawField::Absent { json_path },
    }
}

pub(crate) fn parse_raw_string(value: &RawValue) -> Option<String> {
    serde_json::from_str::<String>(value.get()).ok()
}

pub(crate) fn parse_raw_i64(value: &RawValue) -> Option<i64> {
    serde_json::from_str::<i64>(value.get()).ok()
}

pub(crate) fn parse_raw_u64(value: &RawValue) -> Option<u64> {
    serde_json::from_str::<u64>(value.get()).ok()
}

pub(crate) fn parse_raw_f64(value: &RawValue) -> Option<f64> {
    serde_json::from_str::<f64>(value.get()).ok()
}

pub(crate) fn parse_raw_bool_or_numeric(value: &RawValue) -> Option<bool> {
    serde_json::from_str::<bool>(value.get()).ok().or_else(|| {
        serde_json::from_str::<i64>(value.get()).ok().and_then(|number| match number {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })
    })
}

pub(crate) fn parse_raw_u32_vec(value: &RawValue) -> Option<Vec<u32>> {
    serde_json::from_str::<Vec<u32>>(value.get()).ok()
}

pub(crate) fn observed_raw_shape(value: &RawValue) -> String {
    serde_json::from_str::<JsonShape>(value.get())
        .map_or_else(|_| "unknown".to_string(), |shape| shape.as_str().to_string())
}

fn event_rows<'a>(root: &'a RawOcapRoot<'a>) -> Vec<(usize, &'a RawValue)> {
    let Some(events) = root.raw_value_at("events") else {
        return Vec::new();
    };

    raw_array_items(events).map_or_else(Vec::new, |events| events.into_iter().enumerate().collect())
}

fn connected_event(event: &RawValue, event_index: usize) -> Option<ConnectedEventObservation> {
    let event = raw_array_items(event)?;
    let frame = event.first().and_then(|value| parse_raw_u64(value))?;
    let event_type = event.get(1).and_then(|value| parse_raw_string(value))?;

    if event_type != "connected" {
        return None;
    }

    Some(ConnectedEventObservation {
        event_index,
        frame,
        name: event.get(2).and_then(|value| parse_raw_string(value))?,
        entity_id: event.get(3).and_then(|value| parse_raw_i64(value))?,
        json_path: format!("$.events[{event_index}]"),
    })
}

fn killed_event(event: &RawValue, event_index: usize) -> Option<KilledEventObservation> {
    let event = raw_array_items(event)?;
    let event_type = event.get(1).and_then(|value| parse_raw_string(value))?;

    if event_type != "killed" {
        return None;
    }

    Some(KilledEventObservation {
        event_index,
        frame: event.first().and_then(|value| parse_raw_u64(value)),
        killed_entity_id: event.get(2).and_then(|value| parse_raw_i64(value)),
        kill_info: killed_event_kill_info(event.get(3).copied()),
        distance_meters: event.get(4).and_then(|value| parse_raw_f64(value)),
        json_path: format!("$.events[{event_index}]"),
    })
}

fn killed_event_kill_info(value: Option<&RawValue>) -> KilledEventKillInfo {
    let Some(value) = value else {
        return KilledEventKillInfo::Malformed { observed_shape: "absent".to_string() };
    };

    let Some(values) = raw_array_items(value) else {
        return KilledEventKillInfo::Malformed { observed_shape: observed_raw_shape(value) };
    };

    if values.len() == 1
        && values.first().and_then(|value| parse_raw_string(value)).as_deref() == Some("null")
    {
        return KilledEventKillInfo::NullKiller;
    }

    let [killer_id, weapon] = values.as_slice() else {
        return KilledEventKillInfo::Malformed { observed_shape: observed_raw_shape(value) };
    };

    let Some(killer_entity_id) = parse_raw_i64(killer_id) else {
        return KilledEventKillInfo::Malformed { observed_shape: observed_raw_shape(value) };
    };

    KilledEventKillInfo::Killer {
        killer_entity_id,
        weapon: parse_raw_string(weapon).filter(|weapon| !weapon.is_empty()),
    }
}

fn raw_array_items(value: &RawValue) -> Option<Vec<&RawValue>> {
    serde_json::from_str::<Vec<&RawValue>>(value.get()).ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonShape {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl JsonShape {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
            Self::Array => "array",
            Self::Object => "object",
        }
    }
}

impl<'de> Deserialize<'de> for JsonShape {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonShapeVisitor)
    }
}

struct JsonShapeVisitor;

impl<'de> Visitor<'de> for JsonShapeVisitor {
    type Value = JsonShape;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any valid JSON value")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(JsonShape::Boolean)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(JsonShape::Number)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(JsonShape::Number)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(JsonShape::Number)
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(JsonShape::String)
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(JsonShape::String)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(JsonShape::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(JsonShape::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(JsonShape::Array)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(JsonShape::Object)
    }
}

#[cfg(all(test, not(coverage)))]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use super::*;

    #[test]
    fn decode_compact_root_should_distinguish_malformed_json_from_non_object_root() {
        assert!(matches!(
            decode_compact_root(br#"{"missionName":"ok""#),
            Err(CompactDecodeError::JsonDecode { .. })
        ));
        assert!(matches!(
            decode_compact_root(br#"["not", "object"]"#),
            Err(CompactDecodeError::RootNotObject { .. })
        ));
    }

    #[test]
    fn compact_killed_events_should_keep_malformed_kill_info_auditable() {
        let root = decode_compact_root(
            br#"{
                "events": [
                    [1, "killed", 2, ["null"], 10],
                    [2, "killed", 3, {"bad": true}],
                    [3, "killed", 4, [1, ""]],
                    [4, "ignored", 1]
                ]
            }"#,
        )
        .expect("test root should decode");

        let events = compact_killed_events(&root);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].kill_info, KilledEventKillInfo::NullKiller);
        assert_eq!(
            events[1].kill_info,
            KilledEventKillInfo::Malformed { observed_shape: "object".to_string() }
        );
        assert_eq!(
            events[2].kill_info,
            KilledEventKillInfo::Killer { killer_entity_id: 1, weapon: None }
        );
        assert_eq!(events[2].json_path, "$.events[2]");
    }
}
