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
      "id": "vehicle-score.teamkill-penalty-clamp",
      "target": "parser-core::vehicle_score",
      "description": "teamkill penalty clamp keeps applied weight at least one when raw matrix weight is below one",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_teamkill_penalty_clamp_using_raw_weight_below_one"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "vehicle-score.category-direction",
      "target": "parser-core::vehicle_score",
      "description": "attacker and target vehicle score categories must not be swapped",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_vehicle_score_attacker_and_target_category_swaps"],
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
      "id": "events.null-killer-bounty",
      "target": "parser-core::events",
      "description": "null-killer deaths do not produce bounty inputs",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_null_killer_deaths_producing_bounty_inputs"],
      "accepted_non_applicable_reason": null
    },
    {
      "id": "aggregates.source-refs",
      "target": "parser-core::aggregates",
      "description": "aggregate contributions retain source refs",
      "risk": "high",
      "outcome": "caught",
      "evidence": ["cargo test -p parser-core fault_injection_regressions::fault_injection_regressions_should_catch_aggregate_contributions_without_source_refs"],
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
    "total_cases": 6,
    "by_outcome": {
      "caught": 6
    },
    "high_risk_missed": 0
  }
}
JSON
}

run_deterministic_fallback() {
  cargo test -p parser-core fault_injection_regressions
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
