---
phase: 01
artifact: mismatch-taxonomy-interface-notes
status: ready
---

# Mismatch Taxonomy and Interface Notes

## Taxonomy Summary

Future old-vs-new diff reports must classify every difference with one of the required mismatch categories below and must carry the D-14 impact dimensions:

- `parser artifact` impact
- `server-2 persistence` impact
- `server-2 recalculation` impact
- `UI-visible public stats` impact through `web`

D-12: suspected legacy bugs require a human-review gate before preserving or fixing behavior. D-15: Phase 1 creates notes only and does not change adjacent apps.

## Required Mismatch Categories

| Category | When to use | Parser artifact only? | Can affect `server-2` persistence or recalculation? | Can be UI-visible in public stats through `web`? | User approval required? |
|---|---|---:|---:|---:|---:|
| `compatible` | The old and new outputs match exactly or differ only in an approved non-semantic representation, such as stable ordering or formatting with equivalent values. | Usually yes, if the diff is representation-only. | No, unless persisted hashes or audit text intentionally include the representation. | No. | No. |
| `intentional change` | The new parser intentionally differs from legacy behavior because a prior accepted requirement or decision says the new behavior is correct. | Sometimes. | Yes when persisted artifacts, derived aggregates, or recalculation inputs change. | Yes when public player, squad, rotation, weekly, bounty, or commander stats change. | Yes, through the prior accepted decision or an explicit review note. |
| `old bug preserved` | The old behavior appears wrong, but parity requires preserving it for v1 after review. | Sometimes. | Yes when preserved behavior affects data stored or recalculated by `server-2`. | Yes when preserved behavior appears in public stats through `web`. | Yes, because suspected legacy bugs require human review before preserve/fix decisions. |
| `old bug fixed` | The old behavior appears wrong and the new parser intentionally corrects it after review. | Sometimes. | Yes when the fix changes persisted parser artifacts, events, aggregate inputs, or recalculated stats. | Yes when the fix changes public stats through `web`. | Yes, because suspected legacy bugs require human review before preserve/fix decisions. |
| `new bug` | The new parser or compatibility harness produces behavior that conflicts with the accepted legacy/reference behavior or contract. | It can be parser-artifact-only, but may be broader. | Yes if bad output could be persisted or used for recalculation. | Yes if bad aggregates or API payloads can reach public stats. | No approval to keep it; fix before release or explicitly reclassify through review. |
| `insufficient data` | The available replay, legacy output, generated artifact, logs, or source references do not prove whether old or new behavior is correct. | Sometimes. | Unknown until evidence is collected. | Unknown until evidence is collected. | Yes before choosing preserve/fix when user-visible or persisted impact is possible. |
| `human review` | The diff requires product or domain judgment, suspected legacy-bug triage, unexplained current-vs-regenerated drift, or cross-app impact approval. | Not assumed. | Potentially yes; record the concrete unknowns. | Potentially yes; record the concrete unknowns. | Yes. |

## Impact Dimensions

Every future diff report must include these fields, even when the answer is `no` or `unknown`:

- `parser artifact impact`: whether normalized events, source references, parse status, observed identity, aggregate projections, schema, or deterministic serialization differ.
- `server-2 persistence impact`: whether `server-2` would store different parse artifacts, replay metadata, observed identities, events, player/squad/rotation/commander facts, or parse failure records.
- `server-2 recalculation impact`: whether backend recalculation of ordinary stats, bounty inputs, commander-side stats, or moderation/audit state could change.
- `UI-visible public-stats impact`: whether `web` could show different public player, squad, rotation, weekly, bounty, commander-side, or request-context data through `server-2` APIs.

If an impact is unknown, the diff cannot be treated as `compatible` until the missing evidence is collected.

## Human Review Gate

D-12 requires human review before preserving or fixing suspected legacy bugs. A diff must enter the `human review` category when:

- Current `~/sg_stats/results` differs from regenerated old-parser output and no approved explanation exists.
- Deterministic and default-worker regenerated outputs differ and the cause is not known.
- Legacy source behavior appears internally inconsistent, order-dependent, data-race-prone, or contradictory to accepted product rules.
- A choice can change `server-2` persistence/recalculation or UI-visible public stats through `web`.
- A choice would move old compatibility behavior into the parser core contract.

Human review output should record the chosen final category, rationale, approval source/date, parser artifact impact, `server-2` persistence/recalculation impact, and UI-visible public-stats impact.

## server-2 Interface Notes

`server-2` owns:

- PostgreSQL persistence.
- Canonical player identity and nickname/SteamID history.
- Parse job orchestration and parser completion/failure consumption.
- Aggregate recalculation for player, squad, rotation, commander-side, and bounty outputs.
- Moderation/correction audit and public API shape.

Phase 1 creates notes only. It does not change `server-2` schemas, queues, RabbitMQ/S3 messages, API payloads, canonical identity rules, moderation behavior, or recalculation code.

Future parser artifact or compatibility-layer changes require a `server-2 persistence` and `server-2 recalculation` assessment before implementation. If a diff changes observed identity, event semantics, failure status, aggregate inputs, artifact shape, or contract versioning, adjacent `server-2` docs or code must be checked before the change is accepted.

## web Interface Notes

`web` owns browser UI and consumes public/authenticated APIs from `server-2`. It does not parse replay files and should not consume parser artifacts directly.

Diffs are UI-visible when they can change:

- Public player, squad, rotation, weekly, commander-side, or bounty stats.
- Search/profile data exposed by `server-2`.
- Request/moderation context shown to players, moderators, or admins.
- Generated API types produced from the `server-2` OpenAPI schema.

Phase 1 creates notes only. It does not change `web` screens, routes, generated API types, public UI copy, auth flows, moderation UI, or annual/yearly nomination pages.

## Phase 2 Handoff

Phase 2 should use this taxonomy while defining the parse artifact contract:

- Keep observed identity raw in parser artifacts; canonical identity remains `server-2` ownership.
- Include enough source references for future mismatch reports to determine parser artifact impact.
- Represent structured parse failures and unknown states so `insufficient data` is not caused by avoidable contract gaps.
- Avoid importing legacy game-type filtering, same-name combining, or yearly nomination support into the parser core contract.

## Phase 5 Handoff

Phase 5 should use this taxonomy in the old-vs-new comparison harness:

- Every per-field mismatch must use exactly one required category.
- Every report must include parser artifact impact, `server-2 persistence` impact, `server-2 recalculation` impact, and UI-visible public-stats impact.
- Suspected legacy bugs must remain `human review` until approved as `old bug preserved` or `old bug fixed`.
- Current-vs-regenerated baseline differences from Plan 01-01 remain `human review` until a future diff explains them.
- Annual/yearly nomination outputs remain historical v2 references and should not be included in ordinary v1 parity comparisons.
