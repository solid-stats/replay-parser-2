# Graph Communities — replay-parser-2

_14 communities, named by member analysis (no LLM API used). Source: `.planning/graphs/graph.json`._

| # | Name | Purpose | Key files |
|---|------|---------|-----------|
| 0 | **Entity Normalization** | Normalizes observed unit/player, vehicle, and static weapon entity facts from OCAP rows with backfill and duplicate-slot hinting. | crates/parser-core/src/entities.rs |
| 1 | **Aggregate Table Derivation** | Derives minimal v3 table rows (players, weapons, destroyed vehicles) from normalized combat events. | crates/parser-core/src/aggregates.rs |
| 2 | **Side Facts Normalization** | Normalizes replay-level commander and outcome facts from mission messages and explicit outcome fields. | crates/parser-core/src/side_facts.rs |
| 3 | **Combat Event Normalization** | Normalizes raw killed tuples into deterministic combat events with kill/teamkill/suicide/vehicle semantics. | crates/parser-core/src/events.rs |
| 4 | **Compact OCAP Deserialization** | Borrowed selective OCAP root extraction with compact-first access to top-level facts and event observations. | crates/parser-core/src/raw_compact.rs |
| 5 | **Raw Field Accessors** | Tolerant accessors for selective OCAP replay root fields with defensive shape handling. | crates/parser-core/src/raw.rs |
| 6 | **Parse Artifact Assembly** | Constructs deterministic parser artifacts with success/failure shells and source metadata stripping. | crates/parser-core/src/artifact.rs |
| 7 | **Serde Deserialization** | Custom Serde visitor implementations for compact OCAP root and entity deserialization. | crates/parser-core/src/raw_compact.rs |
| 8 | **Metadata Normalization** | Normalizes replay metadata from observed OCAP top-level fields with frame and time bounds. | crates/parser-core/src/metadata.rs |
| 9 | **Diagnostic Policy** | Policy wrapper and accumulator for deterministic diagnostic emission and limit enforcement. | crates/parser-core/src/diagnostics.rs |
| 10 | **Debug Artifact Construction** | Full deterministic parser-side artifact with complete provenance for audits and debugging. | crates/parser-core/src/debug_artifact.rs |
| 11 | **Legacy Player Eligibility** | Determines legacy player participation in compatibility projections based on backfill and observed fields. | crates/parser-core/src/legacy_player.rs |
| 12 | **Parser Input Types** | Input container and deterministic parser options for replay bytes and caller-provided metadata. | crates/parser-core/src/input.rs |
| 13 | **Public API Entry** | Main library interface exposing public parse functions for standard, debug, and filtered artifact modes. | crates/parser-core/src/lib.rs |
