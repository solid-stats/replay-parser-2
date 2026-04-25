---
phase: 01
slug: legacy-baseline-and-corpus
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-25
---

# Phase 01 - Validation Strategy

Per-phase validation contract for Phase 1 execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| Framework | none - this repository does not have a Rust workspace, CLI, worker, or test suite yet |
| Config file | none |
| Quick run command | `git diff --check` |
| Full suite command | `git diff --check && git status --short` plus the artifact checks below |
| Estimated runtime | under 30 seconds excluding the old-parser baseline run |

## Sampling Rate

- After every task commit: run `git diff --check`, `git status --short`, and the task-specific `test -f` or `rg` command.
- After every plan wave: run all artifact checks listed in this file.
- Before `$gsd-verify-work`: rerun all artifact checks, old-parser source-command preflight, and clean-tree check.
- Max feedback latency: under 30 seconds for docs/artifact checks; baseline execution may run longer and must record its own log path.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-00-01 | 00 | 0 | WF-01, WF-02 | T-01-05 | generated artifacts stay out of commits | doc/git | `rg -n "\\.planning/generated" .gitignore` | no | pending |
| 01-00-02 | 00 | 0 | LEG-01 | T-01-01 | source-command blocker is reproduced or resolved before baseline | command | `cd /home/afgan0r/Projects/SolidGames/replays-parser && source "$HOME/.nvm/nvm.sh" && nvm use --silent v18.14.0 && pnpm run parse -- --help` | no | pending |
| 01-01-01 | 01 | 1 | LEG-01, LEG-02 | T-01-01, T-01-02 | baseline run uses fake HOME and does not mutate real results | artifact | `test -f .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` | no | pending |
| 01-02-01 | 02 | 1 | DOC-01, LEG-03 | T-01-03, T-01-04 | full-history corpus facts are documented from current data | artifact | `test -f .planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` | no | pending |
| 01-03-01 | 03 | 1 | LEG-04 | T-01-03, T-01-05 | legacy filters, config inputs, and output surfaces are documented without secrets | artifact | `test -f .planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` | no | pending |
| 01-04-01 | 04 | 2 | INT-01, INT-02, INT-03, INT-04, LEG-05 | T-01-06 | mismatch categories include parser, server, and web impact | artifact | `test -f .planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` | no | pending |
| 01-05-01 | 05 | 2 | DOC-01, DOC-02, WF-03, WF-04, WF-05 | T-01-05 | README reflects current corpus and workflow requirements | doc | `rg -n "23,473|full-history|AI agents.*GSD|GSD workflow" README.md` | yes | pending |

## Wave 0 Requirements

- [ ] `.gitignore` contains `.planning/generated/`.
- [ ] Canonical old-parser source command is either fixed enough to run preflight, or the blocker is documented with an explicit user decision before any fallback is used as baseline.
- [ ] Generated Phase 1 output path exists only for local heavy artifacts: `.planning/generated/phase-01/`.
- [ ] No command writes directly to real `~/sg_stats/results` or `~/sg_stats/year_results`.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Approve fallback from `pnpm run parse` to `pnpm run parse:dist` as canonical baseline | LEG-01, LEG-02 | This contradicts locked decision D-01 unless explicitly approved | Record the user decision in `baseline-command-runtime.md` and the phase summary before running fallback as baseline |
| Preserve or fix suspected legacy bug | WF-03, WF-04, WF-05, LEG-05 | D-12 requires human review for suspected legacy bugs | Record risk, alternatives, user decision, and chosen handling in `mismatch-taxonomy-interface-notes.md` |

## Security Threat Model

| ID | Threat | Severity | Mitigation | Verification |
|----|--------|----------|------------|--------------|
| T-01-01 | Canonical source command fails but phase proceeds with a different baseline silently | high | Wave 0 must run the source-command preflight and document blocker or explicit approval | `rg -n "pnpm run parse|source-command|parse:dist|fallback" .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` |
| T-01-02 | Old parser overwrites real historical results | high | Run old parser with fake `HOME` and generated run directory | `rg -n "HOME=.*\\.planning/generated/phase-01|fake HOME|results" .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` |
| T-01-03 | Corpus manifest uses stale 3,938-file assumptions | medium | Recount current raw/list/results inputs during execution | `rg -n "23,473|23,456|88,485" .planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` |
| T-01-04 | Replay-list rows are treated as complete corpus truth | medium | Document raw/list discrepancies and malformed/unlisted files | `rg -n "17|unlisted|replaysList" .planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` |
| T-01-05 | Large generated reports, secrets, or relay logs enter git | high | Ignore `.planning/generated/` and commit compact sanitized summaries only | `rg -n "\\.planning/generated" .gitignore && git status --short` |
| T-01-06 | Mismatch taxonomy omits backend/UI impact | medium | Include parser-only, server-2 recalculation/persistence, and UI-visible categories | `rg -n "server-2|web|UI-visible|parser artifact" .planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` |

## Validation Sign-Off

- [x] All planned deliverable types have automated file or text checks.
- [x] Wave 0 covers source-command and generated-artifact prerequisites.
- [x] Manual-only cases are limited to user approval gates required by locked decisions.
- [x] No watch-mode flags are used.
- [x] `nyquist_compliant: true` is set in frontmatter.

Approval: pending execution.
