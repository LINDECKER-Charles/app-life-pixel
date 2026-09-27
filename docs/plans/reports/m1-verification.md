# Milestone 1 verification — rerun of the §5 battery (`feat/rose-atelier-redesign`)

Date: 2026-09-27. Branch: `feat/rose-atelier-redesign`, HEAD `8db9f115d9d806b19f810d50534b1f744fea73cd`
(the commit verified by this report; subsequent waves start from it).

## Working tree check

`git status --porcelain=v1` (and `-uall`) shows only untracked files under `docs/plans/`
(the plan file, `reports/`, lot 0 baseline captures, and this session's log files). No tracked
file is modified, staged, or otherwise different from `8db9f11`. **Working tree is clean; nothing
to commit.**

## Instances reused

- Dev server (port 4260): found down at the start of this session (no listener responded).
  Restarted identically with `LP_ENGINE_PREBUILT=1 npm start --prefix frontend`
  (log: `docs/plans/reports/m1-verif-dev-server-4260.log`), confirmed responding (HTTP 200) within
  1 second, left running for the next agent.
- Local stack (`docker compose ps`): `postgres`, `objectstore`, `mail` were already up and healthy
  (unchanged since `m1-failures.md`, ~6h+ uptime). The `app`-profile `server`/`admin` containers
  were also still up and healthy, occupying ports 8460/8461/8463/8464, exactly as reported by the
  previous milestone. Nothing was started, stopped, or recreated in the stack.

## Command results (same battery, same order as `m1-failures.md`)

| Command | Result | Comparison to m1-failures.md |
|---|---|---|
| `git status` (working tree) | Clean — only `docs/plans/` untracked | Same |
| `npm run lint --prefix frontend` | Passed (exit 0) | Same |
| `npm run test:ci --prefix frontend` | Passed — 89+11+34 files, 453+53+145 = 651 tests, 0 failed | Identical counts |
| `npm run test:tools --prefix frontend` | Passed — 11 suites, 43 tests, 0 failed | Same |
| `npm run build --prefix frontend` | Passed — `dist/app` and `dist/admin` produced; same pre-existing "bundle initial exceeded maximum budget... 965.03 kB" warning, unrelated to the 8 kB per-component style budget | Same |
| `npm run i18n:check --prefix frontend` | Passed (exit 0) — "2 catalogues checked", same informational notice for reused/`design_system.*` keys | Same |
| `npm run i18n:check-bundle --prefix frontend` | Passed (exit 0) — "no catalogue in 47 scripts; 3 files served" | Same |
| `LP_ENGINE_PREBUILT=1 npm run test:engine --prefix frontend` | Passed — 1 file, 16 tests | Same |
| `node design-system/scripts/check.mjs` | Passed — "64 contrast pairs, 146 translated preview keys, 25 local asset references; catalogues and instructions match" | Same |
| `cmp CLAUDE.md AGENTS.md` | Passed (exit 0) — byte-identical | Same |
| `LP_ENGINE_PREBUILT=1 npm run e2e --prefix frontend` (server 4260) | Passed — 6/6 tests | Same |
| `LP_ENGINE_PREBUILT=1 npm run e2e:visual --prefix frontend` (server 4260) | Passed — 5/5 tests (4 capture runs + 1 measurements run) | Same |

Logs for each command are saved next to this report: `m1-verif-lint.log`, `m1-verif-test-ci.log`,
`m1-verif-test-tools.log`, `m1-verif-build.log`, `m1-verif-i18n-check.log`,
`m1-verif-i18n-bundle.log`, `m1-verif-test-engine.log`, `m1-verif-ds-check.log`,
`m1-verif-e2e.log`, `m1-verif-e2e-visual.log`, `m1-verif-e2e-hosted.log`,
`m1-verif-dev-server-4260.log`.

### §5 criteria re-checked by search (lots 0–2 files)

- **No hardcoded hex colours** in the app/shared files migrated by lots 0–2 (icon, empty-state,
  status-banner, menu-button, shell header/engine-notifications/app-footer, account-menu, app.ts,
  the shared style partials): none found outside `_tokens.scss` itself, which is the token
  *definition* file and is expected to hold the hex source values behind the semantic
  `--lp-color-*` custom properties. `canvas-renderer.ts` still keeps its named
  checkerboard/grid constants under the documented exception. Unchanged from `m1-failures.md`.
- **No local `button {}` / `input {}` redefinition** in the same file set, nor anywhere under
  `frontend/projects/shared/src/styles/`: none found. Unchanged.

## Failures remaining

None new. The only remaining failure is the one already reported as `not_run` in
`m1-failures.md`, reproduced identically in this rerun:

### `npm run e2e:hosted --prefix frontend` — not run, same port conflict as before

Ran again to confirm the cause is unchanged. It failed at the same step with the same error:

```
Error: Ports 8460, 8461, 8463, 8464 are in use: stop what listens there first.
  at hosted\stack\network.ts:26
  at expectFree (F:\Git\app-life-pixel\frontend\e2e\hosted\stack\network.ts:26:11)
  at globalSetup (F:\Git\app-life-pixel\frontend\e2e\hosted\global-setup.ts:24:3)
```

Cause: identical to `m1-failures.md` — the pre-existing `life-pixel-server-1` and
`life-pixel-admin-1` containers (Compose `app` profile, started outside any of these sessions,
still healthy, 6h+ uptime) hold ports 8460/8461/8463/8464 that `e2e:hosted`'s own global setup
needs to bind natively for its own `life-pixel-server`/`life-pixel-admin-server` binaries. Per the
task's instructions, these containers were not stopped (stopping shared, already-running
containers that another verification may still depend on is out of scope for this
no-fix verification pass), so the suite could not run. This is the same environment/orchestration
conflict as before, not a regression and not a defect in lots 0–2's code.

**Consequence for §6's risk**: the admin-inherits-tokens check (`console-sign-in.spec.ts`) and the
hosted axe pass are still unverified. Freeing ports 8460–8464 (stopping the `app`-profile
containers, or running in a session where they are not already up) before `npm run build` +
`LP_ENGINE_PREBUILT=1 npm run e2e:hosted` remains the next step for whoever owns that check.

## Known pre-existing gaps outside lots 0–2's scope (re-confirmed unchanged)

Re-checked by search and by the visual-suite's measurements; all identical to `m1-failures.md`,
none of it introduced or fixed by this verification pass (no source file was touched):

- **§5.5 (canvas ≥ 50 % of viewport height)**: `frontend/e2e/visual/output/measurements.json`
  still reports `stageShareOfViewport: 0.485` at 1366×768 and 1024×768 (needs ≥ 0.5). Scoped to
  lots L4/L5.
- **§5.2/§5.4 (no `<main>`, no local `button {}`/`input {}`)**: the same files listed by
  `m1-failures.md` still redefine `button {}`/`input {}` locally or still render a routed `<main>`
  (new-animation form, export dialog files, palette entry form, tag dialog, token dialog,
  import-sprite-sheet form, tool-bar; account/library/mcp/support/tokens pages). None were touched
  by lots 0–2 or by this verification. Scoped to lots 3, 4, 7, 8a/8b, 9.
- **§5.3 (no hardcoded hex colours)**: `account/account-form.scss`, `account/account-page.scss`
  and `support/support.scss` still use `var(--ion-color-danger, #c00)`. Unchanged, same scope as
  above.

## Environment left running

- Dev server on port 4260: restarted (was down), confirmed responding, left running.
- Local stack: `postgres`/`objectstore`/`mail` and the pre-existing `app`-profile
  `server`/`admin` containers all left exactly as found — untouched, still healthy.

## Conclusion

No lot 0–2 work is uncommitted. Every command that ran reproduced its `m1-failures.md` result
exactly (same passes, same counts). The single non-executed control (`e2e:hosted`) failed for the
identical, already-documented environment reason and is not a code defect. No fix was made, per
the mission ("ne corrige rien"). Next waves should start from `8db9f11`.
