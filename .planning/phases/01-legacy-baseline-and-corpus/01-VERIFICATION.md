---
phase: 01-legacy-baseline-and-corpus
verified: 2026-04-25T08:30:34Z
status: passed
score: 9/9 must-haves verified
---

# Phase 1: Legacy Baseline and Corpus Verification Report

**Phase Goal:** Developers can reproduce and inspect the legacy parser and historical data baseline that define v1 parity.
**Verified:** 2026-04-25T08:30:34Z
**Status:** passed

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Repository has a current `README.md` that documents project purpose, scope, current GSD phase, architecture direction, validation data, and the AI + GSD-only development workflow. | VERIFIED | `README.md` contains current phase text, `23,473`, `23,456`, `2026-04-25T04:42:54.889Z`, `88,485`, `14`, `AI agents using the GSD workflow`, and all four Phase 1 dossier names. |
| 2 | Completed GSD/agent work leaves a clean git working tree by committing intended results, never by deleting or discarding completed work; unclear cases are escalated to the user. | VERIFIED | Phase 1 work was committed in atomic docs commits; final verification includes `git status --short` clean check after completion. |
| 3 | Agents challenge instructions that conflict with architecture, current logic, quality standards, maintainability, or proportional scope; they explain risk and safer alternatives. | VERIFIED | `README.md`, `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, and `gsd-briefs/*` all carry the pushback/safer-alternative workflow requirement. |
| 4 | Agents can identify Solid Stats as a multi-project product made of `replay-parser-2`, `server-2`, and `web`, and apply product-wide GSD rules across those projects. | VERIFIED | `README.md`, `.planning/PROJECT.md`, and `gsd-briefs/*` name all three applications and their ownership boundaries. |
| 5 | Agents use risk-based cross-application compatibility checks. | VERIFIED | `README.md`, `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `gsd-briefs/*`, `legacy-rules-output-surfaces.md`, and `mismatch-taxonomy-interface-notes.md` document risk-based compatibility depth and adjacent app impact dimensions. |
| 6 | Developer can run the pinned old parser baseline and see command, commit, runtime versions, environment inputs, worker count, logs, and output locations. | VERIFIED | `baseline-command-runtime.md` records `pnpm run parse`, old parser commits, Node `v18.14.0`, `pnpm@10.33.0`, worker profiles, fake HOME isolation, generated run paths, logs, output counts, and digests. |
| 7 | Developer can inspect a corpus manifest for `~/sg_stats/raw_replays`, `~/sg_stats/results`, and `~/sg_stats/lists/replaysList.json`. | VERIFIED | `corpus-manifest.md` records raw/list/result/year counts, prepared timestamp, malformed files, shape summaries, distribution, and generated profile path. |
| 8 | Developer can inspect documented old parser game-type filters, skip rules, exclusions, and config inputs. | VERIFIED | `legacy-rules-output-surfaces.md` records `mission_name.startsWith`, `startsWith('sgs')`, `2023-01-01`, `empty_replay`, `mace_min_players`, three config JSON files, `nameChanges.csv`, and identity compatibility rules. |
| 9 | Developer can classify any old-vs-new difference using the agreed mismatch taxonomy. | VERIFIED | `mismatch-taxonomy-interface-notes.md` defines `compatible`, `intentional change`, `old bug preserved`, `old bug fixed`, `new bug`, `insufficient data`, and `human review`, with parser artifact, `server-2`, and UI-visible impact fields. |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `.gitignore` | Ignore generated Phase 1 artifacts | VERIFIED | File exists and `git check-ignore -v .planning/generated/phase-01/baseline-runs` reports `.gitignore:1:.planning/generated/`. |
| `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` | Legacy command/runtime dossier | VERIFIED | Exists and substantive; plan artifact check passed. |
| `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` | Full-history corpus manifest | VERIFIED | Exists and substantive; plan artifact check passed. |
| `.planning/phases/01-legacy-baseline-and-corpus/fixture-index.json` | Compact fixture seed index | VERIFIED | Exists; `jq -e 'type == "array" and length >= 5'` passed. |
| `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` | Legacy rule and output-surface dossier | VERIFIED | Exists and substantive; plan artifact check passed. |
| `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` | Mismatch taxonomy and interface notes | VERIFIED | Exists and substantive; plan artifact check passed. |
| `README.md` | Current human-facing project entry point | VERIFIED | Contains current corpus facts, AI+GSD workflow language, `server-2`, `web`, and the four Phase 1 dossier names. |
| `.planning/PROJECT.md` and `gsd-briefs/replay-parser-2.md` | Current project/brief corpus facts | VERIFIED | Both contain `23,473`, `23,456`, and `2026-04-25T04:42:54.889Z`; stale `3,938` text was removed from these current context files. |
| `.planning/phases/01-legacy-baseline-and-corpus/*-SUMMARY.md` | One summary per plan | VERIFIED | Five plan summaries exist. |

**Artifacts:** 9/9 verified

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `.gitignore` | `.planning/generated/phase-01/` | ignore rule | VERIFIED | `gsd-sdk query verify.key-links` reported a regex false negative, but `git check-ignore -v .planning/generated/phase-01/baseline-runs` proves the ignore rule is active. |
| `baseline-command-runtime.md` | old parser `package.json` | canonical script record | VERIFIED | Key-link check found the expected pattern. |
| `baseline-command-runtime.md` | generated baseline runs | recorded generated run paths | VERIFIED | Key-link check found the expected pattern. |
| `baseline-command-runtime.md` | `~/sg_stats/results` | non-destructive real-results statement | VERIFIED | Key-link check found the expected pattern. |
| `baseline-command-runtime.md` | D-08 comparison evidence | current-results comparison | VERIFIED | Key-link check found the expected pattern. |
| `corpus-manifest.md` | generated corpus profile | profile artifact path | VERIFIED | Key-link check found the expected pattern. |
| `fixture-index.json` | `corpus-manifest.md` | profile-derived fixture reasons | VERIFIED | Key-link check found the expected pattern. |
| `legacy-rules-output-surfaces.md` | old parser `getReplays.ts` | game-type filter source reference | VERIFIED | Key-link check found the expected pattern. |
| `legacy-rules-output-surfaces.md` | `server-2`/`web` boundaries | identity and output ownership notes | VERIFIED | Key-link check found the expected pattern. |
| `mismatch-taxonomy-interface-notes.md` | `gsd-briefs/server-2.md` | server impact dimension | VERIFIED | Key-link check found the expected pattern. |
| `mismatch-taxonomy-interface-notes.md` | `gsd-briefs/web.md` | UI-visible impact dimension | VERIFIED | Key-link check found the expected pattern. |

**Wiring:** 11/11 connections verified

## Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| DOC-01 | SATISFIED | - |
| DOC-02 | SATISFIED | - |
| WF-01 | SATISFIED | - |
| WF-02 | SATISFIED | - |
| WF-03 | SATISFIED | - |
| WF-04 | SATISFIED | - |
| WF-05 | SATISFIED | - |
| INT-01 | SATISFIED | - |
| INT-02 | SATISFIED | - |
| INT-03 | SATISFIED | - |
| INT-04 | SATISFIED | - |
| LEG-01 | SATISFIED | - |
| LEG-02 | SATISFIED | - |
| LEG-03 | SATISFIED | - |
| LEG-04 | SATISFIED | - |
| LEG-05 | SATISFIED | - |

**Coverage:** 16/16 Phase 1 requirements satisfied

## Behavioral Verification

| Check | Result | Detail |
|-------|--------|--------|
| Repository test suite | SKIPPED | No `package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, or `requirements.txt` exists in this repo yet. This is expected for Phase 1 because README states no runnable Rust workspace, CLI, worker, or test suite exists yet. |
| Source command preflight | PASSED | `baseline-command-runtime.md` records post-repair `pnpm run parse -- --help` passing under Node `v18.14.0`. |
| Full old-parser baseline profiles | PASSED | `baseline-command-runtime.md` records successful fake-HOME `WORKER_COUNT=1` and default-worker runs with output counts, sizes, and digests. |
| README final coverage command | PASSED | Plan 01-04 summary records the exact `rg`/file existence/`jq`/`git diff --check` verification. |
| Schema drift gate | PASSED | `gsd-sdk query verify.schema-drift 01` returned `{ "valid": true, "issues": [], "checked": 5 }`. |
| Code review gate | SKIPPED | `01-REVIEW.md` records docs-only scope with no source files changed. |

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `.planning/phases/01-legacy-baseline-and-corpus/01-QUESTIONS.html` | 1006 | `placeholder` in textarea UI | Info | Pre-existing generated question UI, not a Phase 1 deliverable. |
| `.planning/phases/01-legacy-baseline-and-corpus/01-QUESTIONS.json` and `.planning/phases/01-legacy-baseline-and-corpus/01-QUESTIONS.html` | multiple | stale 3,938-file question context | Info | Historical discussion artifacts. Current authoritative docs were updated in README, PROJECT, parser brief, corpus manifest, and summaries. |

**Anti-patterns:** 2 informational findings, 0 blockers

## Human Verification Required

None - all Phase 1 deliverables are documentation/evidence artifacts with automated file, grep, JSON, and generated-artifact checks.

## Gaps Summary

**No gaps found.** Phase goal achieved. Ready to proceed.

## Verification Metadata

**Verification approach:** Goal-backward verification from ROADMAP success criteria plus plan-level must-haves.
**Must-haves source:** ROADMAP Phase 1 success criteria and PLAN frontmatter artifacts/key links.
**Automated checks:** Artifact checks, key-link checks, final plan verification commands, schema drift gate, code review gate, `git diff --check`, and JSON validation.
**Human checks required:** 0
**Total verification time:** 6 min

---
*Verified: 2026-04-25T08:30:34Z*
*Verifier: the agent*
