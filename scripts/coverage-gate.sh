#!/usr/bin/env bash
set -euo pipefail

OUTPUT_ROOT="${COVERAGE_OUTPUT_ROOT:-.planning/generated/phase-05/coverage}"
COVERAGE_JOBS="${COVERAGE_JOBS:-1}"
COVERAGE_NICE="${COVERAGE_NICE:-10}"
COVERAGE_CHECK_TIMEOUT_SECONDS="${COVERAGE_CHECK_TIMEOUT_SECONDS:-300}"
COVERAGE_STRICT_TIMEOUT_SECONDS="${COVERAGE_STRICT_TIMEOUT_SECONDS:-1800}"
mkdir -p "$OUTPUT_ROOT"

print_usage() {
  cat <<'USAGE'
usage: scripts/coverage-gate.sh [--check|--strict]

Modes:
  --check   resource-limited smoke coverage summary
  --strict  strict postprocessor gate; requires COVERAGE_ALLOW_HEAVY=1

Strict coverage compiles and runs the full workspace with instrumentation.
To run it intentionally:
  COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict

Resource controls:
  COVERAGE_JOBS defaults to 1.
  COVERAGE_CHECK_TIMEOUT_SECONDS defaults to 300.
  COVERAGE_STRICT_TIMEOUT_SECONDS defaults to 1800.
  COVERAGE_OUTPUT_ROOT overrides generated output directory.
USAGE
}

require_llvm_cov_installed() {
  if cargo llvm-cov --version >"$OUTPUT_ROOT/cargo-llvm-cov.version" 2>"$OUTPUT_ROOT/cargo-llvm-cov.version.err"; then
    return
  fi

  printf '%s\n' "cargo llvm-cov is required; install cargo-llvm-cov before running the coverage gate." >&2
  printf '%s\n' "Install: cargo install cargo-llvm-cov" >&2
  exit 127
}

run_limited() {
  local timeout_seconds=$1
  shift

  local command=("$@")
  if command -v nice >/dev/null 2>&1; then
    command=(nice -n "$COVERAGE_NICE" "${command[@]}")
  fi
  if command -v ionice >/dev/null 2>&1; then
    command=(ionice -c 3 "${command[@]}")
  fi
  if [[ "$timeout_seconds" != "0" ]] && command -v timeout >/dev/null 2>&1; then
    command=(timeout "$timeout_seconds" "${command[@]}")
  fi

  "${command[@]}"
}

run_llvm_cov() {
  local timeout_seconds=$1
  shift

  run_limited "$timeout_seconds" cargo llvm-cov --jobs "$COVERAGE_JOBS" "$@"
}

require_strict_opt_in() {
  if [[ "${COVERAGE_ALLOW_HEAVY:-0}" == "1" ]]; then
    return
  fi

  cat >&2 <<'ERROR'
Strict coverage is resource-heavy and is blocked by default to avoid freezing the workstation.

Use the smoke gate for routine checks:
  scripts/coverage-gate.sh --check

Run strict coverage only when intentionally scheduled:
  COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict
ERROR
  exit 2
}

run_check() {
  require_llvm_cov_installed

  run_llvm_cov "$COVERAGE_CHECK_TIMEOUT_SECONDS" \
    --workspace --all-targets --json --summary-only 2>&1 \
    | tee "$OUTPUT_ROOT/check-summary.json" >/dev/null
  printf '%s\n' "coverage smoke check passed; summary: $OUTPUT_ROOT/check-summary.json"
}

require_threshold_option() {
  local help_text=$1
  local option=$2

  if ! grep -q -- "$option" <<<"$help_text"; then
    printf 'cargo llvm-cov does not support required threshold option %s\n' "$option" >&2
    exit 2
  fi
}

run_strict_gate() {
  require_strict_opt_in
  require_llvm_cov_installed

  local help_text
  help_text=$(cargo llvm-cov --help 2>&1)

  require_threshold_option "$help_text" "--json"
  local report_path="$OUTPUT_ROOT/strict-missing-lines.txt"
  local coverage_json="$OUTPUT_ROOT/coverage.json"
  local gate_summary="$OUTPUT_ROOT/strict-summary.txt"

  if grep -q -- "--fail-under-branches" <<<"$help_text"; then
    printf '%s\n' "cargo llvm-cov supports branch coverage, but branch thresholds are not stable across supported installations." \
      | tee "$OUTPUT_ROOT/threshold-support.txt"
  else
    printf '%s\n' "cargo llvm-cov lacks --fail-under-branches; branch threshold not supported by this installation." \
      | tee "$OUTPUT_ROOT/threshold-support.txt"
  fi

  run_llvm_cov "$COVERAGE_STRICT_TIMEOUT_SECONDS" \
    --workspace --all-targets --json --output-path "$coverage_json" 2>&1 \
    | tee "$OUTPUT_ROOT/coverage-run.log"
  cargo run -p parser-harness --bin coverage-check -- \
    --allowlist coverage/allowlist.toml \
    --coverage-json "$coverage_json" \
    --project-root . \
    --output "$gate_summary" 2>&1 | tee "$report_path"
}

case "${1:-}" in
  --check)
    run_check
    ;;
  ""|--strict)
    run_strict_gate
    ;;
  -h|--help)
    print_usage
    ;;
  *)
    print_usage >&2
    exit 2
    ;;
esac
