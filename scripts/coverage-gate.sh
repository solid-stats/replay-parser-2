#!/usr/bin/env bash
set -euo pipefail

OUTPUT_ROOT=".planning/generated/phase-05/coverage"
mkdir -p "$OUTPUT_ROOT"

if ! cargo llvm-cov --version >"$OUTPUT_ROOT/cargo-llvm-cov.version" 2>"$OUTPUT_ROOT/cargo-llvm-cov.version.err"; then
  printf '%s\n' "cargo llvm-cov is required; install cargo-llvm-cov before running the coverage gate." >&2
  printf '%s\n' "Install: cargo install cargo-llvm-cov" >&2
  exit 127
fi

run_check() {
  cargo llvm-cov --workspace --all-targets --json --summary-only 2>&1 | tee "$OUTPUT_ROOT/check-summary.json" >/dev/null
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

  cargo llvm-cov --workspace --all-targets --json --output-path "$coverage_json" 2>&1 \
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
  "")
    run_strict_gate
    ;;
  *)
    printf 'usage: %s [--check]\n' "$0" >&2
    exit 2
    ;;
esac
