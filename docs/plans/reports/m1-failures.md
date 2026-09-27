# Milestone 1 — test battery for lots 0–2 (`feat/rose-atelier-redesign`)

Date: 2026-09-27. Branch: `feat/rose-atelier-redesign` (HEAD `8db9f11`). No merge was needed:
lots 0–2 committed directly on this branch.

## Working tree check

`git status --porcelain=v1 -uall` shows only untracked files under `docs/plans/` (the plan itself,
`reports/`, and the lot 0 baseline captures) — none of it is tracked by Git, per the task's own
report rule. No modified or staged tracked file, and no uncommitted change belonging to lots 0–2.
**No uncommitted lot work found.**

## Aucun échec

Every command of §5's battery that could run in this environment passed. No test failure, no
lint violation, no build error, and none of the §5 criteria checked here (hardcoded hex colours,
local `button {}`/`input {}` redefinitions) is violated in the files lots 0–2 already migrated.

### Command results

| Command | Result |
|---|---|
| `git status` (uncommitted lot work) | Clean — see above |
| `npm run lint --prefix frontend` | Passed — app/shared/admin all pass, Prettier clean |
| `npm run test:ci --prefix frontend` | Passed — 89+11+34 files, 453+53+145 = 651 tests, 0 failed |
| `npm run test:tools --prefix frontend` | Passed — 12 suites, 43 tests, 0 failed |
| `npm run build --prefix frontend` | Passed — `dist/app` and `dist/admin` produced; only a pre-existing warning ("bundle initial exceeded maximum budget... 965.03 kB", unrelated to the 8 kB per-component style budget) |
| `npm run i18n:check --prefix frontend` | Passed (exit 0) — only the informational "keys no source file names yet" notice for `design_system.*` and a few other reused keys, expected per arbitrage §8 |
| `npm run i18n:check-bundle --prefix frontend` | Passed (exit 0) — "no catalogue in 47 scripts; 3 files served" |
| `LP_ENGINE_PREBUILT=1 npm run test:engine --prefix frontend` | Passed — 1 file, 16 tests |
| `node design-system/scripts/check.mjs` | Passed — "64 contrast pairs, 146 translated preview keys, 25 local asset references; catalogues and instructions match" |
| `cmp CLAUDE.md AGENTS.md` | Passed (exit 0) — byte-identical |
| `LP_ENGINE_PREBUILT=1 npm run e2e --prefix frontend` (server 4260) | Passed — 6/6 tests |
| `LP_ENGINE_PREBUILT=1 npm run e2e:visual --prefix frontend` (server 4260) | Passed — 5/5 tests (4 capture runs + 1 measurements run) |

### §5 criteria already applicable, checked by search

- **No hardcoded hex colours** in files migrated by lots 0–2 (`_tokens.scss`, `_base.scss`,
  `_ionic.scss`, `_accessibility.scss`, the 7 `styles/components/*.scss` partials, `ui/icon`,
  `ui/empty-state`, `ui/status-banner`, `ui/menu/menu-button`, `shell/header`,
  `shell/engine-notifications`, `shell/app-footer`, `account/account-menu.ts`, `app.ts`): none
  found. `canvas-renderer.ts` keeps its named checkerboard/grid constants, per the documented
  exception.
- **No local `button {}` / `input {}` redefinition** in the same set of files, nor anywhere under
  `frontend/projects/shared/src/styles/`: none found.

## Non exécuté

### `npm run e2e:hosted --prefix frontend` — port conflict, not attempted to work around

`npm run build --prefix frontend` ran successfully beforehand (required precondition). The local
stack (`docker compose up -d --wait`) was already up and healthy
(`postgres`, `objectstore`, `mail`, plus `server`/`admin` on the `app` profile, left running by a
previous milestone's admin verification — 6 hours old); it was left running unchanged.

`npm run e2e:hosted` failed immediately in its global setup:

```
Error: Ports 8460, 8461, 8463, 8464 are in use: stop what listens there first.
  at hosted\stack\network.ts:26
  at expectFree (F:\Git\app-life-pixel\frontend\e2e\hosted\stack\network.ts:26:11)
  at globalSetup (F:\Git\app-life-pixel\frontend\e2e\hosted\global-setup.ts:24:3)
```

Cause: `frontend/e2e/hosted/global-setup.ts` builds and starts its own `life-pixel-server` and
`life-pixel-admin-server` Rust binaries natively on ports 8460–8464
(`docs/v1/foundations.md`: "`docker compose up -d --wait` brings the three services up" —
i.e. only `postgres`/`objectstore`/`mail`; `server`/`admin` are behind the `app` Compose profile
and are not part of the standing "pile locale"). Those same ports were already bound by the
`life-pixel-server-1` and `life-pixel-admin-1` containers, started earlier (outside this session)
with `--profile app`, presumably for the lot 1 admin-inheritance check mentioned in §6's risk
table. Stopping those two containers to free the ports was attempted
(`docker compose stop server admin`) and was refused by the Claude Code auto-mode classifier
("Interfere With Workloads"); re-reading the failed run's log afterward was refused for the same
reason. No workaround was attempted, per instructions. The containers and the local stack were
left exactly as found, still healthy.

**Consequence for §6's risk**: the admin-inherits-tokens check
(`console-sign-in.spec.ts`) and the hosted axe pass, both part of `e2e:hosted`, could not run in
this session. They still need a run once the `app`-profile containers are stopped (or a fresh
session where they are not already running) before `npm run e2e:hosted` starts.

**Fix location**: none — this is an environment/orchestration conflict between an
already-running manual verification stack and `e2e:hosted`'s own binaries, not a defect in
lots 0–2's source. Whoever runs `e2e:hosted` next should first confirm nothing else is bound to
8460–8464 (or stop the `app`-profile containers) before starting the suite.

## Known pre-existing gaps outside lots 0–2's scope (carried forward, not new failures)

These were already flagged by the lot 0/1 reports and are unaffected by lots 0–2; not committed as
failures of this milestone, but noted since they touch §5's final criteria:

- **§5.5 (canvas ≥ 50 % of viewport height)** still fails: `frontend/e2e/visual/output/measurements.json`
  reports `stageShareOfViewport: 0.485` at both 1366×768 and 1024×768 (needs ≥ 0.5). Editor layout
  work is scheduled for later lots (L4/L5), not lots 0–2.
- **§5.2/§5.4 (no `<main>`, `button {}`/`input {}` locally)**: several app pages not yet migrated
  by lots 0–2 still redefine `button {}`/`input {}` locally
  (`editor/new-animation/new-animation-form.scss`, `export/export-format-table.scss`,
  `export/export-options-form.scss`, `export/export-snippet.scss`,
  `palette/palette-entry-form.scss`, `timeline/tags/tag-dialog.scss`,
  `tokens/token-dialog.scss`, `tools/import/import-sprite-sheet-form.scss`,
  `tools/tool-bar.scss`) and several routed pages still render a `<main>`
  (`account/*-page.html`/`.ts`, `library/pages/*.ts`, `mcp/agents-page.html`,
  `support/support-page.html`, `support/support-request-page.html`, `tokens/tokens-page.html`).
  None of these files were touched by lots 0–2 (commits `1646e0a`..`8db9f11`); their migration is
  scoped to lots 3, 4, 7, 8a/8b, 9 per the plan's §4.
- **§5.3 (no hardcoded hex colours)**: `account/account-form.scss`, `account/account-page.scss`
  and `support/support.scss` still use `var(--ion-color-danger, #c00)`. Same as above: pre-existing,
  not touched by lots 0–2.

## Environment left running

- Dev server on port 4260: was down at the start of this session; restarted with
  `LP_ENGINE_PREBUILT=1 npm start --prefix frontend` (identical command), logged to
  `docs/plans/reports/dev-server-4260.log`, confirmed responding (HTTP 200), left running.
- Local stack (`docker compose up -d --wait`): confirmed already up and healthy
  (`postgres`, `objectstore`, `mail`), left running; the pre-existing `app`-profile
  `server`/`admin` containers were left untouched (stop attempt refused, see above).
- Host Node is v22.23.2 vs `.nvmrc`'s 24.21.0 (already flagged by lot 0's report); it did not
  prevent any command above from passing.
