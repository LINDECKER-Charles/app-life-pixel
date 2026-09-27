# Wave A — verification

Branch `feat/rose-atelier-redesign`, verified hash **`11c8164`** (root, run on 2026-09-27).
This is the commit wave B branches from.

## 0. Preconditions

- `git status --porcelain=v1`: only `?? docs/plans/` (untracked, not part of wave A). No tracked file
  is modified. Clean.
- `git log -1`: `11c8164 fix(app): expose a single main landmark per page`, on top of the four wave A
  commits reported in `wa-failures.md` (`de4ce61`, `caa50d2`, `6d8af41`) plus this one.
- The 4260 server was **stopped** at the start (`curl` timed out). Restarted the documented way
  (`LP_ENGINE_PREBUILT=1 npm start --prefix frontend`), confirmed `curl -o /dev/null -w '%{http_code}'
  http://localhost:4260` → `200`. Left running for the next agent.
- The local stack (`postgres`, `objectstore`, `mail`, `server`, `admin`) was already up (healthy, 7 h),
  not restarted.

## 1. Same battery as `wa-failures.md` §4, run one at a time

| Command | Result |
|---|---|
| `npm run lint` | passed (app, shared, admin; Prettier clean) |
| `npm run test:ci` | passed: app 528, shared 53, admin 145 tests (app +1 vs. the prior run, consistent with the new single-main test added by `11c8164`) |
| `npm run test:tools` | passed: 43/43 |
| `LP_ENGINE_PREBUILT=1 npm run build` | passed (app + admin, no budget warning) |
| `npm run i18n:check` | passed (exit 0); same pre-existing unused-key notes (`design_system.*`, `cli.*`, dynamic `export.format*`/`updater.*`) |
| `npm run i18n:check-bundle` | passed ("no catalogue in 52 scripts; 3 files served") |
| `LP_ENGINE_PREBUILT=1 npm run test:engine` | passed: 16/16 |
| `node design-system/scripts/check.mjs` | passed (64 contrast pairs, 146 preview keys, 25 assets) |
| `cmp CLAUDE.md AGENTS.md` | identical |
| `npm run e2e` | **passed: 6/6** (the 3 failures in `wa-failures.md` Zone A are fixed) |
| `npm run e2e:visual` | passed: 5/5 |
| `npm run e2e:hosted` | **not run**: same cause as before |

## 2. Zone A failure (export size cells) — fixed

Commit `6d8af41` wraps `{{ rawSize(row) }}` / `{{ gzipSize(row) }}` in a `<span class="value">` inside
`export-format-table.html`, so Angular drops the whitespace-only text nodes and the `<td class="size">`
text content is exactly the size, matching the anchored e2e regex
`/^\d[\d.,]*\s?[a-zA-Z]+$/`. Confirmed by rerunning `npm run e2e`:

- `e2e/editor/accessibility.spec.ts:35` › accessibility › the export dialog — pass
- `e2e/editor/journey.spec.ts:17` › exports: five formats with their sizes, then WASM — pass
- `e2e/editor/keyboard.spec.ts:77` › exports: five formats with their sizes, then WASM — pass

No other regression in the other 3 previously-passing e2e specs.

## 3. C4 gap (nested `<main>`) — fixed

`wa-failures.md` §6 flagged two nested `role="main"` landmarks on C6 pages (`ion-router-outlet` and
`ion-content` both `main`). Commit `11c8164` renames the outlet template ref (`#main` → `#outlet`,
`app.html`), so the outlet no longer carries `role="main"`; `ion-content` remains the page's single
main landmark. The editor page (`editor-page.ts`) instead sets host `role="main"` on itself (it does
not use `ion-content` as its scroll container), so it also carries exactly one main.

Confirmed live in the browser (4260, `document.querySelectorAll('[role="main"], main')`):

| Route | `role="main"` elements found |
|---|---|
| `/sign-in` | 1 — `ION-CONTENT` |
| `/settings` | 1 — `ION-CONTENT` |
| `/editor` | 1 — `LP-EDITOR-PAGE` |
| `/does-not-exist` (404) | 1 — `ION-CONTENT` |

Only console error on each page: `401 /api/v1/auth/session`, expected for a visitor. This matches the
17/17 targeted tests (`app.spec.ts`) reported for `11c8164`, now re-run as part of the full `test:ci`
pass above (app 528/528).

## 4. Checks against criteria C2, C3, C4 (repeat of `wa-failures.md` §6, on `11c8164`)

- **C2** `rg "^\s*(button|input)\s*\{" frontend/projects/app/src`: same 5 matches as before, none
  touched by wave A commits (`git diff 8db9f11 HEAD` on the app project touches nothing in this list):
  `tools/tool-bar.scss:13`, `tools/import/import-image-button.ts:28`,
  `tools/import/import-sprite-sheet-button.ts:30` (lot 4), `palette/palette-entry-form.scss:52`,
  `timeline/tags/tag-dialog.scss:53` (lot 5). `frontend/projects/admin/src/styles.scss:80` also
  redefines `button {}`, out of the C2 scope (`frontend/projects/app/src`) and out of wave A too
  (touched only by the pre-wave-A lot 9 commit `3f5c283`, already in the merge base).
- **C3** Hex colours in `frontend/projects/app/src/**/*.scss`: **none** (`grep` exit 1). Hex colours
  do exist in `shared/src/styles/_tokens.scss` and `admin/src/styles.scss`, both out of scope (token
  definitions, admin keeps its own styling per the plan). Emoji in templates:
  `timeline/layers/layer-list.html:25,47` (`👁`, `✕`, both `aria-hidden`), untouched by wave A (lot 5).
- **C4** `<main` in `frontend/projects/app/src/app/**`: only the mention in `ui/README.md`, no literal
  tag. Single main landmark confirmed on every checked route, see §3 above. `::ng-deep`: none in
  `frontend/projects`.

## 5. Not run, with the cause

- `npm run e2e:hosted`: same as `wa-failures.md` — global setup fails immediately at
  `Error: Ports 8460, 8461, 8463, 8464 are in use: stop what listens there first.`
  (`e2e/hosted/stack/network.ts:26`). The local stack's `server` and `admin` containers hold those
  ports (up 7 h, healthy) and the method forbids stopping shared services to free them. Same hosted
  specs remain not run: the quota fix, the library "Actions for …" menu, the account/support page
  objects, the admin sign-in.
- Drawing on the canvas via `browser_run_code_unsafe`: not attempted this run (not required for
  verification; drawing is covered by `e2e`, which passed 6/6, including the journey and keyboard
  drawing steps).

## 6. Conclusion

No failure remains open from `wa-failures.md`: both reported causes (Zone A export size cells, C4
nested `<main>`) are fixed and reverified on `11c8164`. The only outstanding item is the
environment-imposed `e2e:hosted` port conflict, unchanged since the milestone before wave A.
