# Quick Task 260426-joq: Strict Quality Rules - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

<domain>
## Task Boundary

Add maximally strict stable rules for Rust linting, formatting, and type-safety enforcement in `replay-parser-2`.

This is a local developer tooling task. It does not change parser artifacts, `server-2` integration messages, identity ownership, persistence ownership, or UI-visible behavior.

</domain>

<decisions>
## Implementation Decisions

### Strictness Level
- Use strict stable tooling only. Do not require nightly Rust or unstable rustfmt options.

### Enforcement Surface
- Add repo-level and CI-ready enforcement: workspace lint policy, rustfmt config, cargo aliases, README commands, and GSD artifacts.
- Do not add GitHub Actions in this task.

### Breakage Policy
- If stricter rules reveal current violations, fix them now instead of leaving known failures.

</decisions>

<specifics>
## Specific Ideas

- Current pinned toolchain is Rust `1.95.0` with `rustfmt` and `clippy`.
- Baseline `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass before tightening.
- `clippy::pedantic`, `clippy::nursery`, and `clippy::cargo` currently reveal missing package metadata, missing public API docs, missing `#[must_use]`, missing `const fn`, and one direct float comparison in tests.

</specifics>

<canonical_refs>
## Canonical References

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/research/SUMMARY.md`
- `AGENTS.md`

</canonical_refs>
