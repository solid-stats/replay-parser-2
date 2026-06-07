---
quick_id: 260509-rff
date: 2026-05-09
status: passed
---

# Verification

Commands run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p parser-core
cargo test --workspace
cargo doc --workspace --no-deps
git diff --check
cargo run -p parser-cli -- parse --output /tmp/sg-ks-mission-message-artifact.json --pretty --replay-id ks-sample ~/sg_stats/raw_replays/2026_04_24__23_48_19__2_ocap.json
jq '.side_facts' /tmp/sg-ks-mission-message-artifact.json
```

Result:

- All Rust format, clippy, tests, and docs gates passed.
- Real raw replay parse emitted two observed commander facts from event
  `$.events[242]` and inferred `winner_side: east` from
  `Победа КС: [SHK]Sota. Поражение КС: [JTF2]Bas`.
- No unrelated project or server files were changed.

