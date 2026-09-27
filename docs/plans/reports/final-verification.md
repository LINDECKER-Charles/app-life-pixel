# Final verification — Rose Atelier redesign

Root, branch `feat/rose-atelier-redesign`, HEAD `3a08c04` (fixes at `f71e5ac` plus the
verification docs update `3a08c04` from this session). Battery re-run on the same single
instances: dev server 4260 (code at `f71e5ac`, which `3a08c04` does not change) and the
already-running local stack. Logs: `docs/plans/reports/final2-*.log`; measurements:
`final2-measurements.json`; screenshots: `frontend/e2e/visual/output/screens/` (ignored by Git;
two copied for evidence into `docs/plans/reports/captures/final2-*.png`).

## Preliminary checks

- `git status --porcelain=v1`: only `docs/plans/` (untracked, never committed, as required).
  No tracked file is modified outside this session's own commit.
- `find i18n-pending -type d`: only `i18n-pending/` itself (holds `README.md`); no
  `i18n-pending/<lot>/` directory remains from any wave or worktree agent.
- Dev server 4260: down at the start (`curl` → `000`). Restarted identically with
  `LP_ENGINE_PREBUILT=1 npm start --prefix frontend` (`final2-dev-server-4260.log`), answered 200
  on the first check, left running.
- Local stack: the five containers (`postgres`, `objectstore`, `mail`, `server`, `admin`) were
  already `Up (healthy)`; not touched.

## Battery (§5), re-run in series

| Command | Result | Log |
| --- | --- | --- |
| `npm ci --prefix frontend` | not re-run: `frontend/node_modules` already installed, no `package.json` change since the last final battery | — |
| `npm run lint --prefix frontend` | pass | `final2-lint.log` |
| `npm run test:ci --prefix frontend` | pass: app 582, shared 53, admin 145 | `final2-testci.log` |
| `npm run test:tools --prefix frontend` | pass: 43 | `final2-tools.log` |
| `npm run build --prefix frontend` (`LP_ENGINE_PREBUILT=1`) | pass, no budget warning | `final2-build.log` |
| `npm run i18n:check --prefix frontend` | pass | `final2-i18n.log` |
| `npm run i18n:check-bundle --prefix frontend` | pass | `final2-i18nb.log` |
| `npm run test:engine --prefix frontend` (`LP_ENGINE_PREBUILT=1`) | pass: 16 | `final2-engine.log` |
| `npm run e2e --prefix frontend` (`LP_ENGINE_PREBUILT=1`) | pass: 10 | `final2-e2e.log` |
| `npm run e2e:hosted --prefix frontend` | **not run** (environment): `Error: Ports 8460, 8461, 8463, 8464 are in use: stop what listens there first.` — held by the stack's `server`/`admin` containers, which the method forbids stopping | `final2-hosted.log` |
| `npm run e2e:visual --prefix frontend` (`LP_ENGINE_PREBUILT=1`) | pass: 5, screenshots and measurements refreshed | `final2-visual.log` |
| `node design-system/scripts/check.mjs` | pass: 64 contrast pairs, 146 preview keys, 25 assets | `final2-ds.log` |
| `cmp CLAUDE.md AGENTS.md` | identical | — |

`test:ci` rose from 576 to 582 app tests versus the first final battery (the zone A/C/D/E/F fix
commits added targeted tests); every other count matches. No new failure appeared anywhere in the
battery; the only non-pass is the same environmental `e2e:hosted` block as the first final run.

## The 12 criteria, reassessed

| # | Status | Evidence |
| --- | --- | --- |
| 1 | Met, with `e2e:hosted` not run (cause above, unchanged) | battery table above |
| 2 | Met | unchanged since the first final battery (no button/input redefinition; not re-run, no relevant file touched) |
| 3 | Met | unchanged (no hex literal outside `canvas-renderer.ts`'s named constants, `palette-color.ts`'s artwork default and `engine/testing/`'s mock palette; no emoji) |
| 4 | Met for the pages reachable without the hosted stack | unchanged; `e2e:visual` measurements confirm every such page still scrolls to its end at 390 × 640 |
| 5 | Met | `final2-measurements.json`: 1366 × 768 and 1024 × 768 stage 405 px (0.527 of the window), artwork fits |
| 6 | Met | `e2e` (10/10): active layer/frame, shortcuts, zoom/grid/onion-skin unit tests in `test:ci` |
| 7 | Met | `save-state*.spec.ts` pass within `test:ci` |
| 8 | Met | `journey.spec.ts` and `keyboard.spec.ts` pass within `e2e` |
| 9 | Met, no more layout defect blocking it | `i18n:check` pass; fr screenshots at 1024/390 px show no truncated label; the zone C welcome/view-bar overlap is fixed (see below) |
| 10 | Met | unchanged: light/dark/system/reduced-motion checked live in the first final battery, inside an open `ion-modal`; not re-run live this session, no relevant file touched since |
| 11 | Met | `cmp` identical; `AGENTS.md` cites `design-system/`; `docs/v1/editor.md` unchanged since the last check |
| 12 | Filled, all six recorded defects now fixed and re-checked live | `design-system/docs/verification.md`, "Review matrix for the editor (final re-check)"; NVDA/VoiceOver/TalkBack still not run (human review); admin/skip-link forced-colours issue outside this redesign's files still open |

## Live re-check of the six zone fixes (this session, Chromium via Playwright, port 4260)

All six defects recorded in `docs/plans/reports/final-failures.md` (zones A–F) were reproduced as
fixed by direct interaction, not only by reading test files:

- **Zone A (400 % zoom)**: at 320 × 200 CSS px, `lp-editor-page` is now 200 px tall (was 0 px).
  `ion-app` scrolls the shell; `.editor`'s own grid scrolls to its full content
  (`scrollTop` reaches 745 of 745 px). Also checked the large-text path: at 1280 × 800 with the
  root font size at 200 %, `.editor` scrolls (`scrollHeight` 2372 px in a 537 px box) — and at the
  viewport-shrink equivalent of a real 200 % browser zoom (640 × 400), the same grid scrolls to
  its bottom too. Root-font-size and viewport-shrink remain simulations of the browser's own
  zoom/text-size setting, not the setting itself.
- **Zone B (forced colours)**: "Exporter", with a document open and `forcedColors: 'active'`
  emulated, computes to `color: rgb(0,0,0)` on `background-color: rgb(255,255,255)` with a black
  border (ButtonText on ButtonFace), no longer blank. The admin's `.primary` button and both
  shells' skip link were **not** re-checked: they are outside this redesign's attributed files
  (see "Remaining for the maintainer" below).
- **Zone C (editor layout and welcome)**: `frontend/e2e/visual/output/screens/fr-light/editor-welcome-1024.png`
  (copied to `docs/plans/reports/captures/final2-editor-welcome-1024-fr-light.png`) shows the
  welcome card ending above the view bar's row ("800 %", "Pixel —", "Crayon" fully visible below
  it), not covering it as in the first observation.
- **Zone D (canvas redraw after import)**: covered by `canvas.spec.ts`'s new behaviour test
  (in `test:ci`) and by the live semantics check below, which used "Créer" to open a document and
  interact with the rail without any stale-canvas symptom.
- **Zone E (hidden import inputs exposed)**: the accessibility tree now lists exactly one named
  button per import control ("Importer une image", "Importer une planche de sprites"); each
  control's second, previously exposed element is `[aria-hidden]`.
- **Zone F (Export enabled with no document)**: on the welcome (no animation open), "Exporter" is
  `[disabled]`, matching "Enregistrer".

## Branches

- Integration branch: `feat/rose-atelier-redesign` (this session's HEAD: `3a08c04`).
- The seven lot branches, each with commits, none merged into the integration branch by this
  session (a merge was out of this mission's scope):
  `feat/redesign-account-pages`, `feat/redesign-admin-console`, `feat/redesign-editor-layout`,
  `feat/redesign-flow-dialogs`, `feat/redesign-inspector-timeline`,
  `feat/redesign-library-settings`, `feat/redesign-tools-canvas`.
- `feat/v1-integration` also exists in this repository; it predates this plan's lot branches and
  was not created or touched by this session.

## What remains for the maintainer

- Merge the integration branch (and, if intended, the seven lot branches) into `dev`; this
  session did not merge or push anything.
- Run `e2e:hosted` once ports 8460–8464 are free (stop or reconfigure the long-running
  `server`/`admin` containers first, outside this method's scope), and review the library/visitor
  hosted axe runs.
- Fix the remaining forced-colours defect outside this redesign's attributed files: the admin's
  `.primary` button and the skip links of both the app and admin shells, which still show
  HighlightText on Highlight the way "Export" did before its fix.
- Human review: NVDA on Windows for the editor, welcome, tabs, view bar status and dialogs;
  VoiceOver/TalkBack for a mobile surface; Windows contrast themes and the browser's real text-size
  setting (200 %/400 %) on a physical machine, since this session's root-font-size and
  viewport-shrink checks are simulations; coarse-pointer touch targets (44 px rule).
- Decide whether to keep the account/support/tokens/agents/project pages' criterion-4 scroll
  check as a follow-up, since they need an authenticated session and were not measured by
  `e2e:visual` (unchanged limitation from the first final battery).

## Left as found

- Dev server 4260: left running (as required), serving `f71e5ac`/`3a08c04` (docs-only commit
  layered on top, no app code change).
- Local stack: left untouched, `Up (healthy)`.
- No branch created, merged, deleted or pushed by this session.
