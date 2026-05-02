//! Selected-input old-vs-new comparison logic.
// coverage-exclusion: reviewed Phase 05 comparison fallback branches are allowlisted by exact source line.

use std::collections::BTreeMap;

use parser_contract::{
    identity::EntitySide,
    minimal::{
        KillClassification, MinimalDestroyedVehicleRow, MinimalKillRow, MinimalPlayerRow,
        MinimalWeaponRow,
    },
};
use serde_json::{Map, Value, json};

use crate::report::{
    ComparisonBaseline, ComparisonFinding, ComparisonInput, ComparisonReport, ImpactAssessment,
    ImpactLevel, MismatchCategory, ReportValidationError,
};

/// Compares two selected saved JSON artifacts and returns a structured report.
///
/// # Errors
///
/// Returns [`ComparisonError::InvalidJson`] when either input is not JSON, or
/// [`ComparisonError::InvalidReport`] when report invariants are violated.
pub fn compare_artifacts(
    old_label: impl Into<String>,
    old_json: &[u8],
    new_label: impl Into<String>,
    new_json: &[u8],
) -> Result<ComparisonReport, ComparisonError> {
    let old_label = old_label.into();
    let new_label = new_label.into();
    let old_value = parse_json("old", old_json)?;
    let new_value = parse_json("new", new_json)?;
    compare_values(old_label, &old_value, new_label, &new_value)
}

fn compare_values(
    old_label: String,
    old_value: &Value,
    new_label: String,
    new_value: &Value,
) -> Result<ComparisonReport, ComparisonError> {
    let baseline = baseline_from_old_label(&old_label);
    let baseline_is_drift = baseline.is_current_vs_regenerated_drift();
    let old_view = comparison_view(old_value);
    let new_view = comparison_view(new_value);
    let findings = selected_surfaces()
        .iter()
        .map(|surface| compare_surface(surface, &old_view, &new_view, baseline_is_drift))
        .collect();

    ComparisonReport::new(
        baseline,
        vec![ComparisonInput::new("old", old_label), ComparisonInput::new("new", new_label)],
        findings,
    )
    .map_err(ComparisonError::InvalidReport)
}

fn parse_json(side: &'static str, bytes: &[u8]) -> Result<Value, ComparisonError> {
    serde_json::from_slice(bytes).map_err(|source| ComparisonError::InvalidJson { side, source })
}

fn baseline_from_old_label(old_label: &str) -> ComparisonBaseline {
    if labels_current_vs_regenerated_drift(old_label) {
        ComparisonBaseline {
            old_profile: old_label.to_owned(),
            old_command: format!("saved artifact: {old_label}"),
            worker_count: Some(1),
            source: "current_vs_regenerated_drift".to_owned(),
            diagnostic_only: false,
        }
    } else {
        ComparisonBaseline::saved_artifact(old_label)
    }
}

fn labels_current_vs_regenerated_drift(label: &str) -> bool {
    let label = label.to_ascii_lowercase();
    label.contains("current") && label.contains("regenerated") && label.contains("drift")
}

fn comparison_view(root: &Value) -> Value {
    let mut view = root.clone();

    if has_selected_legacy_surfaces(&view) {
        return view;
    }

    let Some(derived) = derive_legacy_view_from_minimal(root) else {
        return view;
    };
    let Some(object) = view.as_object_mut() else {
        return view;
    };

    for (key, value) in derived {
        let _ = object.entry(key).or_insert(value);
    }

    view
}

fn has_selected_legacy_surfaces(root: &Value) -> bool {
    root.get("legacy").and_then(Value::as_object).is_some_and(|legacy| {
        legacy.contains_key("player_game_results") && legacy.contains_key("relationships")
    }) && root
        .get("bounty")
        .and_then(Value::as_object)
        .is_some_and(|bounty| bounty.contains_key("inputs"))
}

fn derive_legacy_view_from_minimal(root: &Value) -> Option<Map<String, Value>> {
    let tables = MinimalComparisonTables::from_root(root)?;

    let mut derived = Map::new();
    drop(derived.insert(
        "legacy".to_owned(),
        json!({
            "player_game_results": legacy_player_game_results(&tables),
            "relationships": legacy_relationships(&tables),
        }),
    ));
    drop(derived.insert("bounty".to_owned(), json!({ "inputs": bounty_inputs(&tables) })));

    Some(derived)
}

#[derive(Debug, Clone)]
struct MinimalComparisonTables {
    players: Vec<MinimalPlayerRow>,
    weapons: Vec<MinimalWeaponRow>,
    kills: Vec<MinimalKillRow>,
    destroyed_vehicles: Vec<MinimalDestroyedVehicleRow>,
}

impl MinimalComparisonTables {
    fn from_root(root: &Value) -> Option<Self> {
        Some(Self {
            players: rows(root, "players")?,
            weapons: optional_rows(root, "weapons")?,
            kills: optional_rows(root, "kills")?,
            destroyed_vehicles: optional_rows(root, "destroyed_vehicles")?,
        })
    }

    fn weapon_name(&self, weapon_id: Option<u32>) -> Option<&str> {
        weapon_id.and_then(|id| {
            self.weapons.iter().find(|weapon| weapon.id == id).map(|weapon| weapon.name.as_str())
        })
    }
}

fn rows<T>(root: &Value, key: &str) -> Option<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(root.get(key)?.clone()).ok()
}

fn optional_rows<T>(root: &Value, key: &str) -> Option<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    root.get(key)
        .map_or_else(|| Some(Vec::new()), |value| serde_json::from_value(value.clone()).ok())
}

fn legacy_player_game_results(tables: &MinimalComparisonTables) -> Value {
    let players = player_refs(tables);
    let deaths_by_teamkills = deaths_by_teamkills(&tables.kills);
    let destroyed_vehicle_counts = destroyed_vehicle_counts(&tables.destroyed_vehicles);

    Value::Array(
        tables
            .players
            .iter()
            .map(|player| {
                let player_ref = players
                    .by_source_entity_id
                    .get(&player.source_entity_id)
                    .cloned()
                    .unwrap_or_else(|| PlayerComparisonRef {
                        player_id: player_id(player.source_entity_id),
                        source_entity_id: player.source_entity_id,
                        compatibility_key: player_id(player.source_entity_id),
                        observed_entity_ids: vec![player.source_entity_id],
                        observed_name: None,
                        side: None,
                    });
                let deaths_by_teamkill =
                    *deaths_by_teamkills.get(&player_ref.source_entity_id).unwrap_or(&0);
                let vehicle_kills = player
                    .vehicle_kills
                    .max(*destroyed_vehicle_counts.get(&player_ref.source_entity_id).unwrap_or(&0));

                json!({
                    "compatibility_key": player_ref.compatibility_key,
                    "observed_entity_ids": player_ref.observed_entity_ids,
                    "observed_name": player_ref.observed_name,
                    "side": side_name(player_ref.side),
                    "kills": player.kills,
                    "killsFromVehicle": player.kills_from_vehicle,
                    "vehicleKills": vehicle_kills,
                    "teamkills": player.teamkills,
                    "isDead": player.deaths > 0,
                    "isDeadByTeamkill": deaths_by_teamkill > 0,
                    "deaths": {
                        "total": player.deaths,
                        "byTeamkills": deaths_by_teamkill,
                    },
                    "kdRatio": kd_ratio(
                        player.kills,
                        player.teamkills,
                        player.deaths,
                        deaths_by_teamkill,
                    ),
                    "killsFromVehicleCoef": kills_from_vehicle_coef(
                        player.kills,
                        player.kills_from_vehicle,
                    ),
                    "score": score(1, player.kills, player.teamkills, deaths_by_teamkill),
                    "totalPlayedGames": 1,
                })
            })
            .collect(),
    )
}

fn deaths_by_teamkills(kills: &[MinimalKillRow]) -> BTreeMap<i64, u64> {
    let mut counts = BTreeMap::<i64, u64>::new();

    for kill in kills {
        if kill.classification != KillClassification::Teamkill {
            continue;
        }
        if let Some(victim_id) = kill.victim_source_entity_id {
            *counts.entry(victim_id).or_default() += 1;
        }
    }

    counts
}

fn destroyed_vehicle_counts(
    destroyed_vehicles: &[MinimalDestroyedVehicleRow],
) -> BTreeMap<i64, u64> {
    let mut counts = BTreeMap::<i64, u64>::new();

    for destroyed in destroyed_vehicles {
        if let Some(attacker_id) = destroyed.attacker_source_entity_id {
            *counts.entry(attacker_id).or_default() += 1;
        }
    }

    counts
}

fn legacy_relationships(tables: &MinimalComparisonTables) -> Value {
    let players = player_refs(tables);
    let mut relationships = RelationshipBuckets::default();

    for kill in &tables.kills {
        match kill.classification {
            KillClassification::EnemyKill => {
                relationships.add(
                    "killed",
                    kill.killer_source_entity_id,
                    kill.victim_source_entity_id,
                    &players,
                );
                relationships.add(
                    "killers",
                    kill.victim_source_entity_id,
                    kill.killer_source_entity_id,
                    &players,
                );
            }
            KillClassification::Teamkill => {
                relationships.add(
                    "teamkilled",
                    kill.killer_source_entity_id,
                    kill.victim_source_entity_id,
                    &players,
                );
                relationships.add(
                    "teamkillers",
                    kill.victim_source_entity_id,
                    kill.killer_source_entity_id,
                    &players,
                );
            }
            KillClassification::Suicide
            | KillClassification::NullKiller
            | KillClassification::Unknown => {}
        }
    }

    json!({
        "killed": relationships.rows("killed"),
        "killers": relationships.rows("killers"),
        "teamkilled": relationships.rows("teamkilled"),
        "teamkillers": relationships.rows("teamkillers"),
    })
}

#[derive(Debug, Default)]
struct RelationshipBuckets {
    rows_by_relationship: BTreeMap<String, BTreeMap<String, RelationshipRow>>,
}

impl RelationshipBuckets {
    fn add(
        &mut self,
        relationship: &str,
        source_entity_id: Option<i64>,
        target_entity_id: Option<i64>,
        players: &PlayerRefs,
    ) {
        let Some(source) = source_entity_id.and_then(|id| players.by_source_entity_id.get(&id))
        else {
            return;
        };
        let Some(target) = target_entity_id.and_then(|id| players.by_source_entity_id.get(&id))
        else {
            return;
        };
        let row_key =
            format!("{}|{}|{}", relationship, source.compatibility_key, target.compatibility_key);
        let row = self
            .rows_by_relationship
            .entry(relationship.to_owned())
            .or_default()
            .entry(row_key)
            .or_insert_with(|| RelationshipRow::new(relationship, source.clone(), target.clone()));
        row.count += 1;
    }

    fn rows(&self, relationship: &str) -> Value {
        Value::Array(
            self.rows_by_relationship
                .get(relationship)
                .into_iter()
                .flat_map(|rows| rows.values())
                .map(RelationshipRow::to_value)
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
struct RelationshipRow {
    relationship: String,
    source: PlayerComparisonRef,
    target: PlayerComparisonRef,
    count: u64,
}

impl RelationshipRow {
    fn new(relationship: &str, source: PlayerComparisonRef, target: PlayerComparisonRef) -> Self {
        Self { relationship: relationship.to_owned(), source, target, count: 0 }
    }

    fn to_value(&self) -> Value {
        json!({
            "relationship": self.relationship,
            "source_player_id": self.source.player_id,
            "source_entity_id": self.source.source_entity_id,
            "source_compatibility_key": self.source.compatibility_key,
            "source_observed_entity_ids": self.source.observed_entity_ids,
            "source_observed_name": self.source.observed_name,
            "target_player_id": self.target.player_id,
            "target_entity_id": self.target.source_entity_id,
            "target_compatibility_key": self.target.compatibility_key,
            "target_observed_entity_ids": self.target.observed_entity_ids,
            "target_observed_name": self.target.observed_name,
            "count": self.count,
        })
    }
}

fn bounty_inputs(tables: &MinimalComparisonTables) -> Value {
    let players = player_refs(tables);

    Value::Array(
        tables
            .kills
            .iter()
            .filter(|kill| kill.classification == KillClassification::EnemyKill)
            .filter_map(|kill| {
                let killer = player_ref_for_kill_player(&players, kill.killer_source_entity_id)?;
                let victim = player_ref_for_kill_player(&players, kill.victim_source_entity_id)?;

                Some(json!({
                    "killer_player_id": killer.player_id,
                    "killer_source_entity_id": killer.source_entity_id,
                    "killer_compatibility_key": killer.compatibility_key,
                    "killer_side": side_name(killer.side),
                    "victim_player_id": victim.player_id,
                    "victim_source_entity_id": victim.source_entity_id,
                    "victim_compatibility_key": victim.compatibility_key,
                    "victim_side": side_name(victim.side),
                    "weapon": tables.weapon_name(kill.weapon_id),
                    "attacker_vehicle_entity_id": kill.attacker_vehicle_entity_id,
                    "attacker_vehicle_class": kill.attacker_vehicle_class,
                }))
            })
            .collect(),
    )
}

#[derive(Debug, Clone)]
struct PlayerComparisonRef {
    player_id: String,
    source_entity_id: i64,
    compatibility_key: String,
    observed_entity_ids: Vec<i64>,
    observed_name: Option<String>,
    side: Option<EntitySide>,
}

#[derive(Debug, Clone, Default)]
struct PlayerRefs {
    by_player_id: BTreeMap<String, PlayerComparisonRef>,
    by_source_entity_id: BTreeMap<i64, PlayerComparisonRef>,
}

fn player_refs(tables: &MinimalComparisonTables) -> PlayerRefs {
    let mut refs = PlayerRefs::default();

    for player in &tables.players {
        let player_id = player_id(player.source_entity_id);
        let player_ref = PlayerComparisonRef {
            player_id: player_id.clone(),
            source_entity_id: player.source_entity_id,
            compatibility_key: player
                .compatibility_key
                .clone()
                .unwrap_or_else(|| player_id.clone()),
            observed_entity_ids: vec![player.source_entity_id],
            observed_name: player.observed_name.clone(),
            side: player.side,
        };
        drop(refs.by_player_id.insert(player_id, player_ref.clone()));
        drop(refs.by_source_entity_id.insert(player.source_entity_id, player_ref));
    }

    refs
}

fn player_ref_for_kill_player(
    players: &PlayerRefs,
    source_entity_id: Option<i64>,
) -> Option<PlayerComparisonRef> {
    source_entity_id.and_then(|id| players.by_source_entity_id.get(&id).cloned())
}

fn player_id(source_entity_id: i64) -> String {
    format!("entity:{source_entity_id}")
}

const fn side_name(side: Option<EntitySide>) -> Option<&'static str> {
    match side {
        Some(EntitySide::East) => Some("east"),
        Some(EntitySide::West) => Some("west"),
        Some(EntitySide::Guer) => Some("guer"),
        Some(EntitySide::Civ) => Some("civ"),
        Some(EntitySide::Unknown) => Some("unknown"),
        None => None,
    }
}

fn kd_ratio(kills: u64, teamkills: u64, deaths_total: u64, deaths_by_teamkills: u64) -> f64 {
    let deaths_without_teamkills = deaths_total.abs_diff(deaths_by_teamkills);
    let total = replay_count_difference(kills, teamkills);

    if deaths_without_teamkills == 0 {
        return signed_count_to_f64(total);
    }

    round_2(signed_count_to_f64(total) / unsigned_count_to_f64(deaths_without_teamkills))
}

fn score(total_played_games: u64, kills: u64, teamkills: u64, deaths_by_teamkills: u64) -> f64 {
    let total_score = replay_count_difference(kills, teamkills);
    let games_count = replay_count_difference(total_played_games, deaths_by_teamkills);

    if games_count <= 0 {
        return signed_count_to_f64(total_score);
    }

    round_2(signed_count_to_f64(total_score) / signed_count_to_f64(games_count))
}

fn kills_from_vehicle_coef(kills: u64, kills_from_vehicle: u64) -> f64 {
    if kills == 0 || kills_from_vehicle == 0 {
        return 0.0;
    }

    round_2(unsigned_count_to_f64(kills_from_vehicle) / unsigned_count_to_f64(kills))
}

fn round_2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn replay_count_difference(left: u64, right: u64) -> i64 {
    u64_to_i64_saturating(left).saturating_sub(u64_to_i64_saturating(right))
}

fn u64_to_i64_saturating(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn signed_count_to_f64(value: i64) -> f64 {
    let magnitude = unsigned_count_to_f64(value.unsigned_abs());
    if value.is_negative() { -magnitude } else { magnitude }
}

fn unsigned_count_to_f64(value: u64) -> f64 {
    u32::try_from(value).map_or_else(|_| f64::from(u32::MAX), f64::from)
}

const fn selected_surfaces() -> [SelectedSurface; 5] {
    [
        SelectedSurface::new("status", &["status"]),
        SelectedSurface::new("replay", &["replay"]),
        SelectedSurface::new("legacy.player_game_results", &["legacy", "player_game_results"]),
        SelectedSurface::new("legacy.relationships", &["legacy", "relationships"]),
        SelectedSurface::new("bounty.inputs", &["bounty", "inputs"]),
    ]
}

fn compare_surface(
    surface: &SelectedSurface,
    old_root: &Value,
    new_root: &Value,
    baseline_is_drift: bool,
) -> ComparisonFinding {
    let old_value = surface.extract(old_root);
    let new_value = surface.extract(new_root);
    let category = classify_values(old_value, new_value, baseline_is_drift);

    ComparisonFinding::new(
        surface.name,
        None,
        category,
        impact_for_surface(surface),
        old_value.cloned().unwrap_or(Value::Null),
        new_value.cloned().unwrap_or(Value::Null),
    )
}

fn classify_values(
    old_value: Option<&Value>,
    new_value: Option<&Value>,
    baseline_is_drift: bool,
) -> MismatchCategory {
    if baseline_is_drift {
        return MismatchCategory::HumanReview;
    }

    match (old_value, new_value) {
        (Some(old), Some(new)) if old == new => MismatchCategory::Compatible,
        (Some(_), Some(_)) => MismatchCategory::HumanReview,
        _ => MismatchCategory::InsufficientData,
    }
}

fn impact_for_surface(surface: &SelectedSurface) -> ImpactAssessment {
    if surface.is_projection() {
        return ImpactAssessment::new(
            ImpactLevel::Yes,
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
            ImpactLevel::Unknown,
        );
    }

    ImpactAssessment::new(
        ImpactLevel::Yes,
        ImpactLevel::Unknown,
        ImpactLevel::Unknown,
        ImpactLevel::Unknown,
    )
}

#[derive(Debug, Clone, Copy)]
struct SelectedSurface {
    name: &'static str,
    path: &'static [&'static str],
}

impl SelectedSurface {
    const fn new(name: &'static str, path: &'static [&'static str]) -> Self {
        Self { name, path }
    }

    fn extract<'a>(&self, root: &'a Value) -> Option<&'a Value> {
        let mut current = root;
        for segment in self.path {
            current = current.get(*segment)?;
        }

        Some(current)
    }

    fn is_projection(&self) -> bool {
        matches!(self.path, ["summaries", "projections", ..])
    }
}

/// Comparison harness failures.
#[derive(Debug, thiserror::Error)]
pub enum ComparisonError {
    /// One side of the comparison was not valid JSON.
    #[error("{side} artifact is not valid JSON: {source}")]
    InvalidJson {
        /// Compared side label.
        side: &'static str,
        /// JSON parser error.
        source: serde_json::Error,
    },
    /// The produced report violated report invariants.
    #[error(transparent)]
    InvalidReport(#[from] ReportValidationError),
}
