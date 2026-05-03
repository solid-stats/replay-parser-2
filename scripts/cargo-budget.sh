#!/usr/bin/env bash
set -euo pipefail

status=0
cargo "$@" || status=$?

prune_status=0
scripts/prune-cargo-target.sh || prune_status=$?

if (( status != 0 )); then
  exit "$status"
fi

exit "$prune_status"
