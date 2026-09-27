# Lot 10 — final battery, criteria and remaining failures

Branch `feat/rose-atelier-redesign`, root. Battery run on `e1f36e7` plus this lot's changes, since
committed as `fb8cbab` (e2e), `3a0f1fd` (docs) and `80c2f4a` (design-system docs); serially, on
the single instances. `e2e` and `lint` were re-run after the last e2e change. Logs: `docs/plans/reports/final-*.log`;
measurements: `final-measurements.json`; screenshots: `frontend/e2e/visual/output/screens/`
(ignored by Git).

## Services

- Dev server 4260: **down** at the start (`curl` → 000, no listener; its previous log,
  `dev-server-4260.log`, ends normally — the process does not outlive the session that started it).
  Restarted identically with `LP_ENGINE_PREBUILT=1 npm start --prefix frontend`
  (`final-dev-server-4260.log`), answered 200, left running.
- Local stack: the five containers were already `Up (healthy)`; not touched.

## Battery (§5)

| Command | Result | Log |
| --- | --- | --- |
| `lint` | pass | `final-lint.log` |
| `test:ci` | pass: app 576, shared 53, admin 145 (the `save-dialog.spec.ts` failure of wave B did not recur) | `final-testci.log` |
| `test:tools` | pass: 43 | `final-tools.log` |
| `build` (`LP_ENGINE_PREBUILT=1`) | pass, no budget warning | `final-build.log` |
| `i18n:check` | pass | `final-i18n.log` |
| `i18n:check-bundle` | pass | `final-i18nb.log` |
| `test:engine` | pass: 16 | `final-engine.log` |
| `e2e` | pass: 10 (+ accessibility file ×4: 32 passed) | `final-e2e.log`, `final-e2e-a11y-repeat.log` |
| `e2e:hosted` | **not run** (environment): `Error: Ports 8460, 8461, 8463, 8464 are in use: stop what listens there first.` (`e2e/hosted/stack/network.ts:26`) — held by the stack's `server` and `admin` containers, which the method forbids stopping | `final-hosted.log` |
| `e2e:visual` | pass: 5, 135 screenshots | `final-visual.log` |
| `node design-system/scripts/check.mjs` | pass | `final-ds.log` |
| `cmp CLAUDE.md AGENTS.md` | identical | — |

The wave B shortcuts-dialog `color-contrast` failure is explained and fixed on the test side: axe
measured the Ionic dialog while it faded in. `document.getAnimations()` does not list animations
in shadow roots (observed: 2 running in `ion-modal`'s shadow root, 0 listed by the document), so
both axe helpers now wait on every shadow root too (`e2e/editor/animations.ts`).

## The 12 criteria

| # | Status | Evidence |
| --- | --- | --- |
| 1 | Met, with `e2e:hosted` not run (cause above) | battery table |
| 2 | Met | `rg "^\s*(button|input)\s*\{" frontend/projects/app/src` → no match |
| 3 | Met | no hex in `frontend/projects/app/src/**/*.scss`; hex literals in `.ts` only in `canvas-renderer.ts` (named checker/grid constants), `palette/palette-color.ts` (artwork default colour, domain data) and `engine/testing/` (mock palette); no emoji (Extended_Pictographic search empty) |
| 4 | Met for the pages reachable without the hosted stack | no `<main` tag in `app/src/app/**` (only a mention in `ui/README.md`); at 390 × 640 `/sign-in`, `/sign-up`, `/reset-password`, `/verify-email`, `/settings`, `/legal/terms`, 404 reach their end (`/library` redirects to sign-in); account, support, tokens, agents and project pages not measured (need an account) |
| 5 | Met | 1366 × 768 and 1024 × 768: stage 405 px (0.527), artwork fits |
| 6 | Met | e2e: active layer chosen by pointer and keyboard (`aria-current`, "Layer: Layer" in the view bar), active frame, shortcuts b/l/g/[/]/./p/Ctrl+Z/Ctrl+Shift+Z/Ctrl+E; zoom, grid and onion skin by the view bar's unit tests (in `test:ci`) |
| 7 | Met (unit tests) | `save-state.spec.ts`, `save-state-pill.spec.ts` pass |
| 8 | Met | `journey.spec.ts` and `keyboard.spec.ts`: welcome → create → two frames (second on a chosen layer) → preview → export → play |
| 9 | Met, with a layout defect (zone C) | `i18n:check` pass; fr screenshots at 1024 and 390 px: no truncated label |
| 10 | Met | browser: light, dark and system (emulated dark) applied without reload, inside an open `ion-modal`; "Reduce" removes the modal's 250 ms fade/slide (system: opacity 0.27 → 1 over ~260 ms; reduce: 1 at 20 ms); checker `#ffffff`/`#ececee`, canvas surround neutral in both themes |
| 11 | Met | `cmp` identical; `AGENTS.md:40,304` cite `design-system/`; `docs/v1/editor.md` U5/U6 updated |
| 12 | Filled, defects recorded | `design-system/docs/verification.md`, "Review matrix for the editor"; NVDA not run (human review) |

## Failures to fix, by disjoint file zone

### Zone A — app shell: no room left at 400 % zoom

- Check: review matrix, zoom 400 % (viewport 320 × 200) and 320 px.
- Observed: `lp-app-header` 133 px (nav wraps to two rows) + `lp-app-footer` 116 px (three stacked
  links) exceed a 200 px window: `lp-editor-page` gets `clientHeight` 0. At 320 × 640 they keep
  249 px, the editor 391 px. Neither scrolls away (`body` is `overflow: hidden`).
- Probable files: `frontend/projects/app/src/app/app.html`, `app.scss`,
  `shell/app-header.html|.scss`, `shell/app-footer.ts`.

### Zone B — shared styles: primary buttons blank in forced colours

- Check: review matrix, forced colours (`forcedColors: 'active'`).
- Observed: "Export" and "Back to editing" (`lp-button lp-button--primary`) show an empty white
  label: computed `color: rgb(255,255,255)` (HighlightText) on `rgb(55,0,110)` (Highlight), with
  `forced-color-adjust: auto`, so Chromium paints its `Canvas` backplate behind the text.
- Probable files: `frontend/projects/shared/src/styles/_tokens.scss` (forced-colours block,
  `--lp-color-accent: Highlight`, `--lp-color-on-accent: HighlightText`),
  `shared/src/styles/components/_buttons.scss` (e.g. `forced-color-adjust: none` on filled
  variants, or ButtonText/ButtonFace).

### Zone C — editor layout and welcome

- Check: criterion 9 screenshots; review matrix, 200 % text.
- Observed 1: the welcome card is taller than the stage and covers the view bar ("Pixel —" and
  "Crayon" show under its bottom edge) — `screens/fr-light/editor-welcome-1024.png`,
  `editor-welcome-390.png`.
- Observed 2: with the root font size at 200 % (1280 × 800), the desktop grid keeps its layout,
  the rail clips its tools, the stage shrinks to ~110 px, and the lower inspector and timeline are
  unreachable (`lp-editor-page` scrollHeight 1094 for 537 px, no scroll container). Simulation
  caveat: rem media queries do not follow a root style; recheck with the browser text size.
- Probable files: `frontend/projects/app/src/app/editor/editor-page.scss`, `editor/welcome/*`.

### Zone D — canvas not redrawn after "Import image"

- Check: review matrix, keyboard (import).
- Reproduce: create a 32 × 32 animation, "Import image" (Enter), pick `tauri/icons/32x32.png`:
  the frame 1 thumbnail shows the heart, the canvas stays empty (checker and grid only) after 2 s
  and a hover; the next pencil click redraws everything, heart included.
- Probable files: `frontend/projects/app/src/app/canvas/canvas.ts` (redraw trigger) or
  `tools/import/import-image.ts` (how the import reports its change).

### Zone E — import buttons expose their hidden file inputs

- Check: review matrix, semantics.
- Observed: the rail's accessibility tree lists "Import image" and "Import sprite sheet" twice:
  the `<input type="file" class="lp-visually-hidden" tabindex="-1">` stays exposed (as a 1 × 1
  button), enabled even when the visible button is disabled (no animation).
- Probable files: `frontend/projects/app/src/app/tools/import/import-image-button.ts`,
  `import-sprite-sheet-button.ts` (e.g. `aria-hidden="true"` on the input).

### Zone F — Export enabled with no animation

- Check: review matrix, recovery.
- Observed: on the welcome (no document), "Export" is enabled — Save is disabled — and opens the
  export dialog listing every format as "Could not export".
- Probable file: `frontend/projects/app/src/app/export/export-button.ts`.

## Not run, with the cause

- `e2e:hosted`: ports 8460/8461/8463/8464 held by the stack's `server`/`admin` containers
  (see battery). The hosted axe on the library (`library.spec.ts:30`, `visitor.spec.ts:55`) exists
  and was not added to; the helper change in `e2e/hosted/accessibility.ts` is type-checked and
  linted only.
- NVDA / VoiceOver / TalkBack: no screen reader in this environment — human review.
- Coarse-pointer 44 px targets: `(pointer: coarse)` cannot be emulated with the tools available.
- Browser text size 200 % and real Windows contrast themes: approximated (root font size,
  Playwright `forcedColors`).
- `browser_run_code_unsafe` was refused by the session policy; the review used the standard
  browser tools.
