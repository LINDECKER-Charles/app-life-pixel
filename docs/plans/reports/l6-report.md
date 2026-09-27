# Lot 6 — editor on narrow and medium screens

Branch `feat/rose-atelier-redesign`, commits a2ed295 (i18n), d63fff1 (app), 208c0ee (frontend),
e1f36e7 (docs). The 4260 server was down at the start (no journal kept from the previous run);
it was restarted with `LP_ENGINE_PREBUILT=1 npm start --prefix frontend` and left running.

## Delivered

- `editor/inspector-tabs/`: `lp-inspector-tabs` (tablist, roving tabindex, Left/Right wrap,
  Home/End, selection follows focus, `aria-controls`), `InspectorLayout` (matchMedia
  `(max-width: 74.99rem)`), `InspectorTab`; spec with axe.
- `editor-page`: below 75 rem the three inspector sections become tab panels (one shown,
  others `hidden`); 48–74.99 rem a "Panels" button (`aria-expanded`, `aria-controls`) folds the
  inspector; below 48 rem one scrolling column: docbar, stage (max(24rem, 60dvh)), rail
  (wrapping rows), frame strip, tabbed inspector. Inspector moved after the timeline in the DOM.
- `_strip.scss`/`frame-list.scss`: below 48 rem the strip commands form a row above the frames,
  as wide as the timeline region (`100cqi`), the frames pan beneath.
- `tool-bar.scss`: comments and row gap for the narrow rail.
- Visual: tab captures at 768/390 px, folded inspector at 768 px, `overflow` measurement at 320.

## Results

| Check | Result | Log |
| --- | --- | --- |
| lint | pass | l6-lint.log |
| test:ci | pass (app 576, shared 53, admin 145) | l6-testci.log |
| i18n:check | pass | l6-i18n.log |
| build | pass, no budget warning | l6-build.log |
| e2e (editor) | 6 passed | l6-e2e.log |
| e2e:visual | 5 passed, 31 captures per language/theme, 0 page errors | l6-visual.log |

Measurements (l6-measurements.json): stage 405 px at 1366×768 and 1024×768 (0.527 of the
height); at 320×640 page scroll width 320, no element outside stage/timeline sticks out.

## Browser review (http://localhost:4260/editor)

320, 390, 768, 1024, 1440 px; light and dark; fr in the browser, en and fr in the captures.
Zoom 900 %, colour 9, frame 1 of 2, Preview tab and undo kept across 768 → 390 → 320 → 1024 →
1440. A stroke started at 1440 and finished after resizing to 390 was drawn as one line. Tabs:
ArrowRight, End, Home move focus and selection; view state untouched.

## Observations outside the lot

- At 320×640 the app shell's header (nav wraps to two rows) and footer (three stacked links)
  take about 250 px, leaving the editor under 400 px; the shell is outside this lot's files.
- `editor.region.palette` is reused as the Palette tab label; the Layers tab reuses
  `timeline.layers.heading`.
