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

  require_threshold_option "$help_text" "--fail-under-lines"
  require_threshold_option "$help_text" "--fail-under-functions"
  require_threshold_option "$help_text" "--fail-under-regions"
  require_threshold_option "$help_text" "--show-missing-lines"

  local report_path="$OUTPUT_ROOT/strict-missing-lines.txt"
  local strict_args=(
    llvm-cov
    --workspace
    --all-targets
    --text
    --show-missing-lines
    --output-path "$report_path"
    --fail-under-lines 100
    --fail-under-functions 100
    --fail-under-regions 100
  )

  if grep -q -- "--fail-under-branches" <<<"$help_text"; then
    strict_args+=(--branch --fail-under-branches 100)
  else
    printf '%s\n' "cargo llvm-cov lacks --fail-under-branches; branch threshold not supported by this installation." \
      | tee -a "$OUTPUT_ROOT/threshold-support.txt"
  fi

  cargo "${strict_args[@]}" 2>&1 | tee "$OUTPUT_ROOT/strict-summary.txt"
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
