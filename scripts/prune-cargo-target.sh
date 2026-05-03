#!/usr/bin/env bash
set -euo pipefail

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
MAX_MIB="${CARGO_TARGET_MAX_MIB:-6144}"
KEEP_MIB="${CARGO_TARGET_KEEP_MIB:-}"
KEEP_NEWEST_PER_STEM="${CARGO_TARGET_KEEP_NEWEST_PER_STEM:-1}"
DRY_RUN="${CARGO_TARGET_PRUNE_DRY_RUN:-0}"

if [[ -z "$KEEP_MIB" ]]; then
  KEEP_MIB=$((MAX_MIB * 8 / 10))
fi

print_usage() {
  cat <<'USAGE'
usage: scripts/prune-cargo-target.sh

Keeps Cargo build artifacts bounded without deleting the whole target cache.
Run after heavy local builds/tests, or use scripts/cargo-budget.sh as a wrapper.

Environment:
  CARGO_TARGET_DIR                     target directory, default: target
  CARGO_TARGET_MAX_MIB                 prune only when target exceeds this, default: 6144
  CARGO_TARGET_KEEP_MIB                prune down to this size, default: 80% of max
  CARGO_TARGET_KEEP_NEWEST_PER_STEM    hashed deps executables kept per test/bin stem, default: 1
  CARGO_TARGET_PRUNE_DRY_RUN=1         print removals without deleting
USAGE
}

case "${1:-}" in
  -h|--help)
    print_usage
    exit 0
    ;;
  "")
    ;;
  *)
    print_usage >&2
    exit 2
    ;;
esac

if [[ ! "$MAX_MIB" =~ ^[0-9]+$ ]] || [[ ! "$KEEP_MIB" =~ ^[0-9]+$ ]]; then
  printf '%s\n' "CARGO_TARGET_MAX_MIB and CARGO_TARGET_KEEP_MIB must be integer MiB values." >&2
  exit 2
fi

if (( KEEP_MIB > MAX_MIB )); then
  printf '%s\n' "CARGO_TARGET_KEEP_MIB must be <= CARGO_TARGET_MAX_MIB." >&2
  exit 2
fi

if [[ ! -d "$TARGET_DIR" ]]; then
  printf 'cargo target prune: %s does not exist; nothing to prune.\n' "$TARGET_DIR"
  exit 0
fi

size_mib() {
  du -sm "$TARGET_DIR" 2>/dev/null | awk '{ print $1 }'
}

current_size_mib() {
  local size
  size=$(size_mib)
  if [[ -z "$size" ]]; then
    printf '%s\n' 0
  else
    printf '%s\n' "$size"
  fi
}

needs_prune() {
  local size
  size=$(current_size_mib)
  (( size > MAX_MIB ))
}

below_keep_size() {
  local size
  size=$(current_size_mib)
  (( size <= KEEP_MIB ))
}

remove_path() {
  local path=$1
  if [[ ! -e "$path" ]]; then
    return
  fi

  if [[ "$DRY_RUN" == "1" ]]; then
    printf 'would remove %s\n' "$path"
  else
    rm -rf -- "$path"
  fi
}

remove_if_present() {
  local path=$1
  if [[ -e "$path" ]]; then
    printf 'cargo target prune: removing %s\n' "$path"
    remove_path "$path"
  fi
}

delete_file() {
  local path=$1
  if [[ "$DRY_RUN" == "1" ]]; then
    printf 'would remove %s\n' "$path"
  else
    rm -f -- "$path"
  fi
}

delete_hashed_dep_sidecars() {
  local path=$1
  local dep_file="${path}.d"
  delete_file "$path"
  if [[ -f "$dep_file" ]]; then
    delete_file "$dep_file"
  fi
}

prune_coverage_and_incremental() {
  remove_if_present "$TARGET_DIR/llvm-cov-target"

  while IFS= read -r -d '' incremental_dir; do
    remove_if_present "$incremental_dir"
  done < <(find "$TARGET_DIR" -mindepth 2 -maxdepth 3 -type d -name incremental -print0 2>/dev/null)
}

prune_hashed_executables() {
  local deps_dir=$1
  [[ -d "$deps_dir" ]] || return

  local candidates=()
  mapfile -t candidates < <(
    find "$deps_dir" -maxdepth 1 -type f -perm /111 -printf '%T@ %p\n' 2>/dev/null | sort -r -n
  )

  declare -A kept_by_stem=()
  local stale=()
  local entry path base stem
  for entry in "${candidates[@]}"; do
    path=${entry#* }
    base=${path##*/}
    if [[ "$base" =~ ^(.+)-[0-9a-f]{16}$ ]]; then
      stem=${BASH_REMATCH[1]}
      if (( ${kept_by_stem[$stem]:-0} < KEEP_NEWEST_PER_STEM )); then
        kept_by_stem[$stem]=$(( ${kept_by_stem[$stem]:-0} + 1 ))
      else
        stale+=("$entry")
      fi
    fi
  done

  local sorted_stale=()
  mapfile -t sorted_stale < <(printf '%s\n' "${stale[@]}" | sort -n)

  for entry in "${sorted_stale[@]}"; do
    [[ -n "$entry" ]] || continue
    path=${entry#* }
    printf 'cargo target prune: removing stale executable %s\n' "$path"
    delete_hashed_dep_sidecars "$path"
    if below_keep_size; then
      return
    fi
  done
}

prune_oldest_dep_files() {
  local candidates=()
  mapfile -t candidates < <(
    find "$TARGET_DIR" \
      -path '*/deps/*' \
      -type f \
      \( -name '*.rlib' -o -name '*.rmeta' -o -name '*.so' -o -name '*.a' \) \
      -printf '%T@ %p\n' 2>/dev/null | sort -n
  )

  local entry path
  for entry in "${candidates[@]}"; do
    [[ -n "$entry" ]] || continue
    path=${entry#* }
    printf 'cargo target prune: removing old dependency artifact %s\n' "$path"
    delete_file "$path"
    if below_keep_size; then
      return
    fi
  done
}

initial_size=$(current_size_mib)
if (( initial_size <= MAX_MIB )); then
  printf 'cargo target prune: %s MiB <= %s MiB; nothing to prune.\n' "$initial_size" "$MAX_MIB"
  exit 0
fi

printf 'cargo target prune: %s MiB > %s MiB; pruning toward %s MiB.\n' \
  "$initial_size" "$MAX_MIB" "$KEEP_MIB"

prune_coverage_and_incremental
if below_keep_size; then
  printf 'cargo target prune: done, target is %s MiB.\n' "$(current_size_mib)"
  exit 0
fi

while IFS= read -r -d '' deps_dir; do
  prune_hashed_executables "$deps_dir"
  if below_keep_size; then
    printf 'cargo target prune: done, target is %s MiB.\n' "$(current_size_mib)"
    exit 0
  fi
done < <(find "$TARGET_DIR" -mindepth 2 -maxdepth 3 -type d -name deps -print0 2>/dev/null)

prune_oldest_dep_files
final_size=$(current_size_mib)

if (( final_size > MAX_MIB )); then
  printf 'cargo target prune: target is still %s MiB; lower the budget or run cargo clean.\n' "$final_size" >&2
  exit 1
fi

printf 'cargo target prune: done, target is %s MiB.\n' "$final_size"
