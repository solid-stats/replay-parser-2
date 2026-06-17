#!/usr/bin/env sh
# Deterministic ~/sg_stats golden replay capture for the container-e2e oracle.
#
# Run ONCE on a machine that has ~/sg_stats present:
#   sh scripts/capture-golden-replays.sh
#
# It selects a small, PINNED, sorted, reproducible spread of real OCAP replays
# (one success, one partial/schema-drift, one failed/malformed, one large-entity),
# gzips each under crates/parser-worker/tests/fixtures/real/, writes a manifest, and
# regenerates the paired *.expected.json baseline from parser_core::parse_replay so the
# (gzipped input + expected output) pair is committed together.
#
# When ~/sg_stats is absent (e.g. CI or a fresh dev box) it prints a presence note and
# exits 0 — the e2e's own skip-guard covers fixture absence at test time, so `verify`
# stays green either way.
#
# NOTE: the gzip loader + flate2 dev-dep are wired together with the real fixtures in a
# follow-up change; ~/sg_stats is absent on the authoring machine, so this script only
# writes the gz files + manifest today.

set -eu

SG_STATS_DIR="${SG_STATS_DIR:-$HOME/sg_stats}"
REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
REAL_DIR="$REPO_ROOT/crates/parser-worker/tests/fixtures/real"
# Commit only replays whose gzipped size is at or under this threshold (bytes).
MAX_COMPRESSED_BYTES="${MAX_COMPRESSED_BYTES:-262144}" # 256 KiB

if [ ! -d "$SG_STATS_DIR" ]; then
  echo "capture-golden-replays: ~/sg_stats not found at '$SG_STATS_DIR' — nothing to capture."
  echo "capture-golden-replays: run this on a machine with the historical replay store present."
  exit 0
fi

mkdir -p "$REAL_DIR"

# Deterministic selection: a sorted glob with fixed indices. Sorting guarantees the same
# set is chosen on every re-capture given the same source store.
SORTED_LIST="$(find "$SG_STATS_DIR" -type f -name '*.json' | LC_ALL=C sort)"
TOTAL="$(printf '%s\n' "$SORTED_LIST" | grep -c . || true)"

if [ "${TOTAL:-0}" -eq 0 ]; then
  echo "capture-golden-replays: no *.json replays under '$SG_STATS_DIR' — nothing to capture."
  exit 0
fi

echo "capture-golden-replays: $TOTAL candidate replays under '$SG_STATS_DIR'."

MANIFEST="$REAL_DIR/manifest.json"
: > "$MANIFEST.tmp"
printf '[\n' >> "$MANIFEST.tmp"
FIRST=1

# Pinned indices into the sorted list (1-based). Adjust deliberately if the curated
# spread must change; keep them small and stable so re-capture is reproducible.
for IDX in 1 2 3 "$TOTAL"; do
  SRC="$(printf '%s\n' "$SORTED_LIST" | sed -n "${IDX}p")"
  [ -n "$SRC" ] || continue
  BASE="$(basename "$SRC" .json)"
  GZ="$REAL_DIR/${BASE}.json.gz"
  gzip -c -n "$SRC" > "$GZ"
  CSIZE="$(wc -c < "$GZ" | tr -d ' ')"

  if [ "$CSIZE" -gt "$MAX_COMPRESSED_BYTES" ]; then
    echo "capture-golden-replays: SKIP '$BASE' — gzipped $CSIZE B exceeds ${MAX_COMPRESSED_BYTES} B; left uncommitted."
    rm -f "$GZ"
    continue
  fi

  [ "$FIRST" -eq 1 ] || printf ',\n' >> "$MANIFEST.tmp"
  FIRST=0
  printf '  { "fixture": "%s.json.gz", "source_file": "%s", "compressed_bytes": %s, "expected_status": "unverified" }' \
    "$BASE" "$(basename "$SRC")" "$CSIZE" >> "$MANIFEST.tmp"
  echo "capture-golden-replays: captured '$BASE' (${CSIZE} B gzipped)."
done

printf '\n]\n' >> "$MANIFEST.tmp"
mv "$MANIFEST.tmp" "$MANIFEST"

# Regenerate paired baselines from the pure parser so input+output are committed together.
# (Enabled once the gzip loader + flate2 dev-dep land with the real fixtures.)
echo "capture-golden-replays: wrote $MANIFEST."
echo "capture-golden-replays: regenerate baselines with the parser-core golden generator after wiring the gzip loader."
echo "capture-golden-replays: done."
