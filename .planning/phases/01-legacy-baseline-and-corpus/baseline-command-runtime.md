---
phase: 01
artifact: baseline-command-runtime
status: wave-0-gate
---

# Baseline Command and Runtime

## Plan 00 Source Command Gate

This gate records the canonical old-parser source-command preflight required by D-01 and D-02 before any full old-parser baseline run.

| Field | Value |
|-------|-------|
| Legacy repo path | `/home/afgan0r/Projects/SolidGames/replays-parser` |
| Legacy commit | `3392ca2f367a87f6eb59041a239e7ca2519e1ec5` |
| Canonical command | `pnpm run parse` |
| Script mapping | `parse` -> `tsx src/start.ts` |
| Runtime target | `.nvmrc` `v18.14.0` |
| Local pnpm version | `10.26.1` |
| Lockfile hash | `df6c812b390fbb3a604deca8c3cf0c278501f3075a0da90d98248264828f132c` |
| Preflight log path | `.planning/generated/phase-01/baseline-runs/20260425T073300Z-source-preflight/source-preflight.log` |

Preflight command:

```bash
cd /home/afgan0r/Projects/SolidGames/replays-parser
source "$HOME/.nvm/nvm.sh"
nvm use --silent v18.14.0
pnpm run parse -- --help > "/home/afgan0r/Projects/SolidGames/sg-replay-parser-2/.planning/generated/phase-01/baseline-runs/20260425T073300Z-source-preflight/source-preflight.log" 2>&1
```

Initial source command status: FAIL

First actionable error lines:

```text
/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - utils/namesHelper/prepareNamesList.ts:4
import { isEmpty } from 'lodash';
         ^

SyntaxError: The requested module 'lodash' does not provide an export named 'isEmpty'
Node.js v18.14.0
```

No full old-parser baseline was run by Plan 00.

## Plan 00 Gate Decision

Gate decision: source command unblocked; workflow auto-advance selected `repair-source-command` and applied a separate source-runtime compatibility repair before any full baseline run.

Compatibility repair:

| Field | Value |
|-------|-------|
| Legacy repair commit | `5e639fc0af222d198a4d20c402f2c8edb0bdc90d` |
| Repair scope | Replaced Lodash named ESM imports with default Lodash imports plus destructuring. |
| Parser logic change | None intended; this is an import/runtime compatibility repair for the canonical source command. |
| Post-repair preflight log path | `.planning/generated/phase-01/baseline-runs/20260425T073702Z-source-preflight-after-repair/source-preflight-after-repair.log` |

Post-repair preflight command:

```bash
cd /home/afgan0r/Projects/SolidGames/replays-parser
source "$HOME/.nvm/nvm.sh"
nvm use --silent v18.14.0
pnpm run parse -- --help > "/home/afgan0r/Projects/SolidGames/sg-replay-parser-2/.planning/generated/phase-01/baseline-runs/20260425T073702Z-source-preflight-after-repair/source-preflight-after-repair.log" 2>&1
```

Source command status: PASS

Post-repair preflight output:

```text
parse: Run the main replay parsing and statistics pipeline.
Usage: pnpm run parse
```

Validation note: `pnpm run tsc` was attempted after the repair and still fails on pre-existing NodeNext/package typing issues, including missing explicit relative import extensions and `@types/node`/DOM `AbortSignal` conflicts. That failure is not used as the Plan 00 gate because the canonical acceptance check is the source-command `--help` preflight.

## Secondary diagnostic only

Diagnostic command:

```bash
cd /home/afgan0r/Projects/SolidGames/replays-parser
source "$HOME/.nvm/nvm.sh"
nvm use --silent v18.14.0
pnpm run parse:dist -- --help > "/home/afgan0r/Projects/SolidGames/sg-replay-parser-2/.planning/generated/phase-01/baseline-runs/20260425T073300Z-source-preflight/parse-dist-diagnostic.log" 2>&1
```

Diagnostic status: PASS

Diagnostic output:

```text
parse: Run the main replay parsing and statistics pipeline.
Usage: pnpm run parse
```

parse:dist is not the canonical Phase 1 baseline unless the user explicitly approves a fallback override.
