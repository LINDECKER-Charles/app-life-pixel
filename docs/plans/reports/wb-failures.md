# Wave B — integration milestone: failures

Branch `feat/rose-atelier-redesign`, final commit `a3081a7` (2026-09-27).

- Merges (`--no-ff`, no conflict): `feat/redesign-tools-canvas` (lot 4, `96d12a2`) → `17b5abd`;
  `feat/redesign-inspector-timeline` (lot 5, `9063ea1`) → `8d4cbe3`.
- `c598b4f` `feat(i18n)`: `merge-i18n-fragments.mjs` (no `--keep`) merged `l4` and `l5`, applied
  `removed.txt` (`tools.rectangle_filled`, `timeline.layers.hide`), deleted the fragments.
- `a3081a7` `docs`: `docs/v1/editor.md` U1 stage, U2 view bar and cursor, U3 frames, layers,
  tags and preview, U4 palette, tool rail and shortcuts dialog.
- No `frontend` commit: none of the names that lots 4 and 5 changed is used by
  `frontend/e2e/editor/` (layer controls, "Edit/Remove colour N", 1-based tag range fields).
  Every locator the page objects use still resolves. Neither e2e failure below comes from a page
  object.
- Dev server 4260: it was down at the start (no listener). I restarted it with
  `LP_ENGINE_PREBUILT=1 npm start --prefix frontend`; it answers 200 and serves the merged code.
  The Docker stack was up (5 containers).

## Battery (serial, root)

| Command | Result | Log |
|---|---|---|
| `npm run lint` | pass | `wb-lint.log` |
| `npm run test:ci` | pass (app 563, shared 53, admin 145) | `wb-testci.log` |
| `npm run test:tools` | pass (43) | `wb-tools.log` |
| `LP_ENGINE_PREBUILT=1 npm run build` | pass, no budget warning | `wb-build.log` |
| `npm run i18n:check` | pass | `wb-i18n.log` |
| `npm run i18n:check-bundle` | pass | `wb-i18nb.log` |
| `LP_ENGINE_PREBUILT=1 npm run test:engine` | pass (16) | `wb-engine.log` |
| `node design-system/scripts/check.mjs` | pass | `wb-ds.log` |
| `cmp CLAUDE.md AGENTS.md` | identical | — |
| `LP_ENGINE_PREBUILT=1 npm run e2e` | **FAIL**: 2 failed, 4 passed | `wb-e2e.log` |
| `LP_ENGINE_PREBUILT=1 npm run e2e:visual` | pass (5) | `wb-visual.log`, `wb-measurements.json` |
| `npm run e2e:hosted` | **not run** (environment) | `wb-hosted.log` |

## Failures by disjoint file zone

### Zone A — canvas view bar (`frontend/projects/app/src/app/canvas/view-bar/`, `canvas/canvas.*`)

- Command: `npm run e2e`. Test: `e2e/editor/journey.spec.ts:17` "draws, animates, exports and
  plays an animation".
- Extract: `Error: frame 1, pixel 1,1 … Expected [0,0,0,255] Received [0,162,232,255]`
  (`player-page.ts:75`). The pencil stroke never landed, so the blue fill covered it.
- Cause, seen in the trace (`e2e/editor/test-results/journey-…/trace.zip`): the first
  `mouse.move` goes to y = 237.1 and the next ones to y = 221.9 for the same row. When the
  pointer enters the canvas, the view bar's position text changes from "Pixel —" to
  "Pixel 3, 1". Its status line then wraps onto a second row, the drawing surface loses about
  30 px of height, and the centred artwork jumps about 15 px under the pointer in the middle of
  the drag. The screencast frames before and after the move show it at 1280 × 720 in English.
  The same wrap is visible in the fr-light 1024 capture.
- This is a real UX defect (the artwork moves under the cursor), not a page-object issue: the
  view bar must keep a stable height. Possible fixes: a fixed-width or `tabular-nums` position
  field, no wrapping, a reserved row height, or taking the view bar out of the canvas's measured
  box. Likely files: `canvas/view-bar/view-bar.scss` / `view-bar.html`, `canvas/canvas.scss`.

### Zone B — shortcuts dialog (`frontend/projects/app/src/app/tools/shortcuts-help-dialog.*`)

- Command: `npm run e2e`. Test: `e2e/editor/accessibility.spec.ts:42` "the shortcuts dialog".
- Extract: `scrollable-region-focusable (serious): Scrollable region must have keyboard access —
  .lp-dialog__body`.
- Cause: with lot 4's key caps, the list overflows `.lp-dialog__body`, which holds no focusable
  element. A keyboard user cannot scroll it (WCAG 2.1.1). Likely fix: `tabindex="0"` plus an
  accessible name on the scrolling body (or on the list), or a layout that does not scroll.
  Likely files: `tools/shortcuts-help-dialog.html` / `.scss`. Do not change the shared
  `.lp-dialog__body` in `_components.scss` unless other dialogs need the same fix.

## Not run

- `npm run e2e:hosted`: global setup stops at `Error: Ports 8460, 8461, 8463, 8464 are in use:
  stop what listens there first.` (`e2e/hosted/stack/network.ts:26`). The local stack's `server`
  and `admin` containers publish those ports. The method forbids stopping shared services, so
  the cause is the same as in wave A.
- Commit hooks after the merge: none active (`.git/hooks` holds only samples, no
  `core.hooksPath`). There is nothing to replay; lint and Prettier ran on the merged tree and
  passed.

## Plan §5 criteria checked

- **2** pass: `rg "^\s*(button|input)\s*\{"` over `frontend/projects/app/src` gives no match.
- **3** pass: no hex colour in the app's `.scss`. The only colour literals are the named
  `CHECKER_LIGHT`, `CHECKER_DARK` and `GRID_COLOR` in `canvas-renderer.ts`, plus mock palette
  data in `engine/testing/mock-state.ts` (test data). No emoji in the templates.
- **5** pass (`wb-measurements.json`). At 1366 × 768 the stage is 405 px (0.527 of the height),
  the drawing surface 950 × 358, and the artwork fits. At 1024 × 768 the stage is 405 px
  (0.527), the surface 608 × 328, and the artwork fits. The drawing surface alone is under half
  the height because the view bar sits inside the stage. The criterion measures the stage, so
  it holds.
- **6** mostly pass. Zoom −/+/fit, grid, onion skin with before/after, active layer and active
  frame all work by pointer in the browser. The keyboard e2e (`keyboard.spec.ts`) passes. Shortcut
  keys are unchanged: the registered keys in the diff from `11c8164` are the same, and only Shift+G
  now goes through `toggleGrid()`. **Gap**: zone A, because the view bar moves the canvas under
  the pointer, so pointer drawing is unreliable.
- **7** pass by the unit tests (`save-state` specs in `test:ci`). In the browser, "Not saved" /
  « Non enregistrée » is shown for new work in both languages.

## Browser review (http://localhost:4260/editor, 1280 × 720)

Checked in fr, then en: create 16 × 16; add a layer; click it (it becomes "Actif", and the view
bar says "Calque : Calque"); draw a red pixel on it; hide "Calque 1" (it says « Masqué »). Zoom +
(800 → 900 %). Grid off. Add frame; select frame 2; onion skin on shows frame 1 faded. Play
preview: the `<life-pixel>` element plays with `motion=always`. Stop. Export dialog: sizes shown,
GIF marked « Le plus petit » in text. Dark scheme with English via Settings, no reload: correct.
The only console error is the expected guest `401` on `/api/v1/auth/session`.

Other observations (not failures, for the next lots):

- At 1280 × 720 the tool rail clips its last button ("Keyboard shortcuts", only half visible).
- Each import's hidden file input shows up in the accessibility tree as a second button with the
  same name ("Import image" twice, one of them enabled while the visible one is disabled with no
  document), in `tools/import/import-image-button.ts` and its sprite-sheet twin. Consider
  `aria-hidden="true"` on the input.
- A new layer is named "Calque" with no number, the translated default of the moment. It is
  document data, not UI.
- Lot 4 follow-ups still open, outside this milestone's files: the view-bar comment in
  `editor/editor-page.html` and the now unused `.view-bar` area in `editor/editor-page.scss`;
  document `ui/tooltip` in `ui/README.md`.
- `docs/v1/editor.md` now has 408 lines (it had 394). The 400-line limit in `AGENTS.md` targets
  code and is not linted for Markdown; trim it if the rule is meant for docs too.
