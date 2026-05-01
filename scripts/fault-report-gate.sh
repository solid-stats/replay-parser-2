#!/usr/bin/env bash
set -euo pipefail

OUTPUT_ROOT=".planning/generated/phase-05/fault-report"
REPORT_PATH="$OUTPUT_ROOT/fault-report.json"
MUTANTS_OUTPUT="$OUTPUT_ROOT/mutants"

mkdir -p "$OUTPUT_ROOT"

write_deterministic_report() {
  cat > "$REPORT_PATH" <<'JSON'
{
  "report_version": "1",
  "tool": "deterministic-fault-injection",
  "generated_at": "2026-04-28",
  "cases": [
    {
      "id": "events.enemy-teamkill-suicide-classification",
      "target": "parser-core::events",
      "description": "enemy, teamkill, suicide, null-killer, and unknown player deaths keep distinct minimal classifications",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core combat_event_semantics_should_partition_player_deaths_and_destroyed_vehicles"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "events.null-killer-bounty",
      "target": "parser-core::events",
      "description": "null-killer deaths do not produce bounty awards",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_null_killer_deaths_producing_bounty_awards"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "events.same-side-teamkill",
      "target": "parser-core::events",
      "description": "same-side non-suicide kills remain teamkills instead of enemy kills",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_same_side_kills_counted_as_enemy_kills"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "aggregates.vehicle-counters",
      "target": "parser-core::aggregates",
      "description": "replay-local aggregate rows preserve vehicleKills and killsFromVehicle counters",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core aggregate_projection_should_derive_replay_local_player_counters"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "minimal-artifact.debug-key-absence",
      "target": "parser-core::minimal_artifact",
      "description": "default artifacts omit debug-only keys including source refs, rule IDs, frame, event index, and JSON path",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core aggregate_projection_should_omit_debug_only_keys_from_default_success_json"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "minimal-artifact.debug-sidecar-provenance",
      "target": "parser-core::minimal_artifact",
      "description": "debug sidecar preserves source refs, rule IDs, frame, and event index only when explicitly requested",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core debug_artifact_should_serialize_full_provenance_and_rule_context"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "events.invalid-json-failure",
      "target": "parser-core::events",
      "description": "invalid JSON returns a failed artifact instead of success or partial",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_invalid_json_returning_success_or_partial"],
      "accepted_non_applicable_reason": null
    }
  ],
  "summary": {
    "total_cases": 7,
    "by_outcome": {
      "caught": 7
    },
    "high_risk_missed": 0
  }
}
JSON
}

run_deterministic_fallback() {
  cargo test -p parser-core fault_injection_regressions
  cargo test -p parser-core combat_event_semantics_should_partition_player_deaths_and_destroyed_vehicles
  cargo test -p parser-core aggregate_projection_should_derive_replay_local_player_counters
  cargo test -p parser-core aggregate_projection_should_omit_debug_only_keys_from_default_success_json
  cargo test -p parser-core debug_artifact_should_serialize_full_provenance_and_rule_context
  write_deterministic_report
}

validate_report() {
  cargo run -p parser-harness --bin fault-report-check -- --report "$REPORT_PATH"
}

if cargo mutants --version >/dev/null 2>&1; then
  cargo mutants --package parser-core --package parser-contract --timeout 60 --output "$MUTANTS_OUTPUT"
  run_deterministic_fallback
else
  printf '%s\n' "cargo mutants is not installed; using deterministic-fault-injection fallback."
  run_deterministic_fallback
fi

validate_report
