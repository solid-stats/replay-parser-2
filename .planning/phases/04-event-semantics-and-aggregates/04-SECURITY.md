---
phase: 04-event-semantics-and-aggregates
slug: event-semantics-and-aggregates
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-28T10:29:01+07:00
---

# Phase 04 - Security

Per-phase security contract for event semantics and aggregate projection threat mitigation.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| OCAP JSON input to parser-core | Historical replay JSON is untrusted and may contain malformed event/entity shapes. | Raw replay metadata, entities, events, killed tuples, side/outcome fields |
| Parser-core to parse artifact consumers | Normalized events, side facts, diagnostics, and aggregate projections become the auditable contract consumed by later tooling and `server-2`. | Versioned `ParseArtifact` JSON plus schema |
| Parser contract to adjacent apps | Parser output must preserve observed identifiers and contribution evidence without claiming `server-2` or `web` ownership. | Observed entity IDs, legacy compatibility keys, source refs, aggregate inputs |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-04-00-01 | Tampering | contract schema | mitigate | Schema-visible contract tests and committed schema regeneration; `cargo test -p parser-contract` and schema `cmp` passed. | closed |
| T-04-00-02 | Repudiation | aggregate payloads | mitigate | Aggregate contribution refs carry event IDs, source refs, rule IDs, and typed values; contract and aggregate tests passed. | closed |
| T-04-00-03 | Information Disclosure | diagnostics/events | mitigate | Diagnostics and events use paths/source refs, not raw replay snippets; contract diagnostics tests passed. | closed |
| T-04-00-04 | Elevation of Privilege | identity boundary | mitigate | Payloads use observed entity IDs and legacy keys only; canonical identity negative tests and boundary grep passed. | closed |
| T-04-01-01 | Denial of Service | raw event parsing | mitigate | Tolerant killed-event accessors preserve malformed observations and avoid panics; `raw_event_accessors` passed. | closed |
| T-04-01-02 | Repudiation | source refs | mitigate | Event source refs carry frame/event index/path/rule evidence; source-ref and raw accessor tests passed. | closed |
| T-04-02-01 | Tampering | combat classification | mitigate | Combat tests cover enemy kill, teamkill, suicide, null killer, vehicle victim, unknown actor, and malformed tuples. | closed |
| T-04-02-02 | Repudiation | combat source refs | mitigate | Normalized combat events include non-empty source refs with event coordinates and rule IDs. | closed |
| T-04-02-03 | Elevation of Privilege | bounty eligibility | mitigate | Excluded event kinds carry bounty exclusion reasons and do not populate bounty-awarding inputs. | closed |
| T-04-03-01 | Repudiation | aggregate counters | mitigate | Projection rows carry contribution IDs linked to event/source refs; `aggregate_projection` passed. | closed |
| T-04-03-02 | Tampering | bounty inputs | mitigate | Only enemy kill events populate `bounty.inputs`; excluded event kinds are regression-tested. | closed |
| T-04-03-03 | Elevation of Privilege | identity projection | mitigate | Aggregates use observed entity IDs and `legacy_name:` keys, never canonical player IDs. | closed |
| T-04-04-01 | Tampering | vehicle score matrix | mitigate | Issue #13 matrix is implemented with tests for category mapping and low/high weights. | closed |
| T-04-04-02 | Repudiation | vehicle contribution audit | mitigate | Every vehicle score input is an aggregate contribution with event and vehicle/category source refs. | closed |
| T-04-04-03 | Elevation of Privilege | teamkill penalties | mitigate | Penalty weights below 1 are clamped to 1 and raw/applied weights are preserved; `vehicle_score` passed. | closed |
| T-04-05-01 | Spoofing | commander identity | mitigate | Commander facts emit observed actor refs only and no canonical commander/player identity. | closed |
| T-04-05-02 | Tampering | winner extraction | mitigate | Winner parsing accepts only explicit side aliases; unknown/conflicting values produce unknown/diagnostic states. | closed |
| T-04-05-03 | Repudiation | inferred commander candidates | mitigate | Candidate commanders carry confidence, rule ID, and source refs; token-boundary regression tests passed. | closed |
| T-04-06-01 | Tampering | generated schema | mitigate | Fresh schema export matched committed `schemas/parse-artifact-v1.schema.json`. | closed |
| T-04-06-02 | Repudiation | deterministic output | mitigate | Populated artifact determinism, contribution ordering, projection-key ordering, and unset timestamp tests passed. | closed |
| T-04-06-03 | Elevation of Privilege | scope boundaries | mitigate | Boundary grep found only README future-scope documentation and canonical identity negative tests. | closed |

## Accepted Risks Log

No accepted risks.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-28 | 21 | 21 | 0 | Codex |

## Sign-Off

- [x] All threats have a disposition.
- [x] Accepted risks documented in Accepted Risks Log.
- [x] `threats_open: 0` confirmed.
- [x] `status: verified` set in frontmatter.

**Approval:** verified 2026-04-28
