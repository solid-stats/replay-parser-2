# Golden Fixture Manifest

This directory contains the Phase 5 golden fixture coverage manifest. The
manifest intentionally links mostly to the existing focused parser-core fixtures
under `crates/parser-core/tests/fixtures/` instead of copying bulky corpus data.

Fixture paths in `manifest.json` are relative to the `crates/parser-core` crate.
Each entry records the covered category, requirement IDs, decision IDs, expected
parse status, expected parser features, provenance, and downstream impact notes.

Full corpus material, regenerated old-parser outputs, benchmark samples, and
generated comparison reports stay outside git under `.planning/generated/phase-05/`.
