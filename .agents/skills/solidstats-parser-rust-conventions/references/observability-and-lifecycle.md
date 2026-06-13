# Observability & resource lifecycle

> **Parity contract — TS mirror.** §K/§L/§M mirror `solidstats-shared-backend-ts-standards`
> §Z (log hygiene) / §AA (diagnosability) / §AB (resource lifecycle) in Rust form. §M omits the
> DB-rows leg of §AB (parser writes no database rows — not applicable). When doctrine changes,
> update **both sides in the same pass** — or leave a `TODO(#issue)` on the side you could not
> update. Editing either side without the other is a review finding.

## §K. Log hygiene

Structured logging via `tracing` is the operational interface of the parser worker. This section
keeps spans and events *hygienic*; §L covers whether they are *diagnosable*.

- **Structured fields, no format-string concatenation** — instrument events with named fields:
  `tracing::warn!(replay_id = %id, job_id = %job, "message")`, never
  `tracing::warn!("replay {} failed: {}", id, e)`. Concatenating identity into the message string
  makes the log unsearchable and unqueryable. Correlate every span and event in `parser-worker` by
  `replay_id` / `job_id` so the full parse flow is followable end-to-end. [🟡 on concat —
  evidence gate: a `tracing::*!` call whose message embeds a variable by `{}` interpolation instead
  of a named field]
- **Log levels mean things** — `warn!` = an unexpected state needing attention; `debug!` = an
  expected code path. A missing optional field in a partial-parse result is `debug!`, not `warn!`.
  A nack that routes a message to the DLX is `warn!`. [🟡 on misuse — evidence gate: `warn!` inside
  a branch that handles a normal/recoverable case documented as expected]
- **State transitions are instrumented** — a function that changes a job's lifecycle state
  (accepted → parsing → success/failure, ack/nack, DLX route) contains at least one
  `info!`/`warn!` event or a span recording the transition. [🟡 if absent — evidence gate: a
  function name matching `*ack*`, `*nack*`, `*complete*`, `*fail*`, `*transition*` with zero
  `tracing::*!` in the body]
- **No PII, no whole-struct dumps** — log the identifier (`replay_id`, `job_id`, an offset), not
  the full deserialized payload; never pass an entire struct as a field unless it explicitly
  implements a redacting `Display`/`Debug`. [🟡 — evidence gate: a `?field` or `%field` tracing
  field whose type is a large domain struct or contains a user payload]
- **Span hierarchy** — the top-level job handler in `parser-worker` opens a span
  (`tracing::info_span!("parse_job", replay_id = %, job_id = %)`) so every child event inherits
  correlation automatically; child spans for S3 download and the `parser-core` parse call are
  entered inside it.

---

## §L. Log diagnosability

§K keeps logs hygienic; §L is the operational question: **when the worker misbehaves in staging or
production, do the tracing events alone let you follow the normal flow and pinpoint what failed
and why** — without reattaching a debugger?

Two guardrails bound the scope:

- **Never add a `match`-arm or wrapper solely for logging.** §L applies only to error paths that
  **already exist** — existing `match` arms that return `Err`, existing `?`-propagations in a `let`
  binding, or existing explicit `if let Err` guards — and to genuine flow inflection points.
  Demanding a new error-handling layer "for logging" is a layering violation.
- **The boundary is already logged.** `parser-worker` has two logging boundaries: **(1)** the
  top-level job handler (the function that receives the lapin `Delivery` and calls `parser-core`)
  and **(2)** the lapin ack/nack path (where the final outcome is committed). `parser-cli` has one:
  the exit-code path at `main`. Do **not** flag an inner function for "no logs" when its error
  propagates unmodified to one of these boundaries — the boundary records it. §L targets only what
  the boundary **cannot see**: which branch ran, *why* an error was raised, and errors
  **swallowed before they ever reach the boundary**.

**Swallowed errors — silent `let _ = …` / `.ok()` on a Result that carries failure [🟠]:** a
`Result` discarded with `let _ =` or `.ok()` (without logging) drops failure information
permanently. The worst case is a silently degraded parse — the failure vanishes and a downstream
symptom is all you get.
```rust
// VIOLATION — parse failure disappears: nothing logged, nothing propagated
let _ = state.mark_complete(job_id);

// CORRECT — record why before continuing
if let Err(e) = state.mark_complete(job_id) {
    tracing::warn!(err = %e, job_id = %job_id, "failed to mark job complete, state may be inconsistent");
}
```
Also flag: a `match` arm on an `Err` variant that does neither `tracing::*!` nor `return Err(…)` /
`?`. Evidence gate: `let _ =` / `.ok()` / `.unwrap_or_default()` (without a preceding comment
explaining why the error is intentionally ignored) on a `Result`-typed expression, or a `match Err`
arm with an empty or trivially silent body.

**Error source chain preserved [🟡]:** when you log inside an error path, pass the **error value**
as a structured field — `tracing::error!(err = %e, "msg")` or `err = ?e` for `Debug` — so the full
`source()` chain is visible. Logging only `e.to_string()` discards the chain. The parser already
requires `#[source]` on `thiserror` variants (§D); the logging side must match by passing `err = ?e`
(or `%e` when `Display` is the right surface) to preserve the chain. Evidence gate: a
`tracing::error!/warn!` inside an error branch that references the error only via string
interpolation in the message, not as a named field.

**Identifying context in errors [🟡]:** an error raised or logged from a branch that *has* an
identifier in scope (`replay_id`, `job_id`, a byte offset, a field name) must carry it — as a
`tracing` field on the event or in the error variant's payload. §D already requires the offending
field/offset in `thiserror` variants; this rule extends that to the *logging* side: the event
emitted on that path must also name the identifier. Cross-reference §D — don't duplicate the rule,
enforce the logging half here. Evidence gate: a `tracing::*!` call on an error path where an
identifying binding (`replay_id`, `job_id`, `offset`, `field`) is in scope but not named as a
field. Don't flag generic leaf functions that have no identifier in scope.

**Upstream failures are diagnosable [🟡]:** when an S3 or RabbitMQ call fails, the log event on
that path captures enough to tell whether it was *us* or *them* — the **error code / HTTP status**
and, for S3 SDK errors, the **request ID** — before propagating. A dependency's 5xx storm must be
visible in the worker's own logs.
```rust
// S3 download path
tracing::warn!(
    err = ?e,
    replay_id = %replay_id,
    s3_request_id = s3_request_id_from_metadata(&e),
    "S3 download failed"
);
return Err(WorkerError::S3Download(e));
```
Evidence gate: an S3/lapin error path that propagates the error with `?` or wraps it without
logging the SDK error detail (status/request-id for S3; channel error kind for lapin).

**Happy-path flow is legible [🔵]:** beyond what the boundary already emits, the meaningful
*decisions* the worker makes should be followable — which arm of the parse-outcome match ran, that
an S3 download was issued, that a message was routed to the DLX vs requeued vs acked. One
`debug!`/`info!` at each real inflection point. Apply **sparingly**: don't flag trivial field
accessors or one-line guards.
```rust
tracing::info!(replay_id = %id, job_id = %job, status = "success", "parse complete, acking");
```
Evidence gate: a non-trivial branch (a business decision, an S3 call, a DLX route, a
skip/`continue` in a loop over deliveries) with no `tracing::*!` on the arm taken.

---

## §M. Resource lifecycle — unbounded growth

The `parser-worker` is a **long-lived process**. Resources that outlive a single job accumulate
with every delivery and are never reclaimed. These leaks rarely appear in short-lived tests — review
is the main gate.

A leak finding needs all **three legs**: **(1) the resource outlives the job delivery, (2) there is
an unbounded write path, (3) nothing ever removes or caps it.** State which legs you found.

**In-memory accumulation on worker state (RAM) [🟠 / 🔴 on hot path]:**
- No field on a long-lived struct (the worker state, a connection holder, any singleton in
  `parser-worker`) that is an unbounded `Vec` or `HashMap` with a per-job write and no
  corresponding removal or capacity cap. A collection allocated *inside* the job handler and dropped
  at its end is scoped and fine.
  ```rust
  // VIOLATION — worker struct field, keyed per replay, never evicted
  struct WorkerState {
      seen_replays: HashMap<ReplayId, Instant>,  // grows forever
  }

  // CORRECT — bounded by construction
  struct WorkerState {
      // bounded LRU or pruned on completion:
      recent_jobs: BoundedCache<JobId, JobOutcome>,
  }
  ```
  Evidence gate: a `HashMap`/`Vec` field on a struct that survives across deliveries, with a
  `.insert(…)` / `.push(…)` on the per-job path and **no** `.remove(…)` / `.clear()` / capacity
  bound visible in the change.

**Unbounded channels [🟠]:** prefer `tokio::sync::mpsc::channel(N)` (bounded) over
`unbounded_channel()` for any channel that receives messages on a per-job path in the worker.
An unbounded channel between the delivery loop and a processing task lets memory grow without
backpressure if the consumer stalls. Bound the channel to a small multiple of the lapin
`prefetch_count`. Evidence gate: `tokio::sync::mpsc::unbounded_channel()` used on a
delivery/processing path without a documented reason.

**Temp files — RAII cleanup [🟠]:** any temporary file created during a parse job (scratch
deserialization, intermediate artifact, downloaded S3 object written to disk) must be cleaned up
via RAII. Prefer the `tempfile` crate (`NamedTempFile` / `TempDir`) so the `Drop` impl removes it
even on panic or early return. Never write to a manually named path and rely on a `finally`-style
explicit delete — that path is skipped on early return. Evidence gate: a `std::fs::File::create` or
`tokio::fs::File::create` for an explicitly named temporary path with no corresponding
`std::fs::remove_file` in a `Drop` impl or via a `tempfile` guard.

**S3 multipart upload aborted on failure [🟠]:** if the worker ever writes a result artifact back
to S3 via multipart upload, a failed mid-upload must call `abort_multipart_upload` — otherwise the
partial parts accumulate as orphaned S3 storage that is never reclaimed. Wrap the upload in a guard
or an `on_error` path that issues the abort. Evidence gate: a multipart upload initiation
(`create_multipart_upload`) with no matching `abort_multipart_upload` on the error path.

Severity defaults to **🟠**. Escalate to **🔴** when the leak is on the hot delivery path and
grows fast enough to exhaust the resource under normal traffic (e.g. an unbounded per-job
in-memory cache on the worker struct). A genuinely low-frequency path can be **🟡**.
