# Wave A — integration and test milestone

Branch `feat/rose-atelier-redesign`, final hash **`caa50d2`** (root, run on 2026-09-27).

## 1. Merges

Five `git merge --no-ff` commits, in this order:

| Lot | Branch | Head merged | Result |
|---|---|---|---|
| 3 | `feat/redesign-editor-layout` | `9831eb4` | clean (`640800f`) |
| 7 | `feat/redesign-flow-dialogs` | `c66652e` | 1 conflict, resolved |
| 8a | `feat/redesign-library-settings` | `df97ea2` | clean |
| 8b | `feat/redesign-account-pages` | `a283afc` | clean |
| 9 | `feat/redesign-admin-console` | `3f5c283` | clean (`b82cedf`) |

Conflict: `frontend/projects/app/src/app/library/save/save-dialog.spec.ts`. Both lots rewrote the
`saveButton()` helper. The merge keeps lot 7's version, scoped to the host (`:scope > button`,
which still matches lot 3's native `button.lp-button`), plus lot 3's `SaveFlow` import and its
`status() === 'saved'` assertion. `test:ci` passes on the result.

Commit hooks: none in the repo (`.git/hooks` holds only `.sample` files, and `core.hooksPath` is
unset), so there was nothing to replay. Lint and Prettier ran on the whole merged tree (below).

## 2. i18n

`node frontend/tools/merge-i18n-fragments.mjs` merged l3, l7, l8a and l8b, then deleted the fragments
(commit `de4ce61`, scope `i18n`). No key had different values in two lots. Keys removed:
`library.save.quota.message` (l7) and `library.animations.empty` (l8a); neither is still referenced.
`i18n:check` passes; its only notes are pre-existing unused keys (`design_system.*`, `cli.*`, and the
dynamic `export.format*` and `updater.*` keys).

## 3. Shared page objects

`frontend/e2e/hosted/pages/outlet.ts` needs no change. Lots 3, 8a and 8b already updated
`hosted-editor.ts`, `library-page.ts`, `account-pages.ts` and `support-pages.ts`.
One hosted spec still referred to a retired text, and is fixed in commit `caa50d2`:
`frontend/e2e/hosted/quota.spec.ts` looked for "Your storage is full: … bytes are used." (the removed
`library.save.quota.message`). It now looks for the new usage line, `/[\d,]+ of 200,000 bytes used\./`,
inside the storage-full dialog. Prettier and `tsc -p tsconfig.e2e.json` pass. The fix is **not run**:
e2e:hosted could not run (see §4).

## 4. Test suite (§5), run one at a time on the merged code

The 4260 server was **stopped** at the start. It was restarted the documented way
(`LP_ENGINE_PREBUILT=1 npm start --prefix frontend`, which includes copy-i18n and copy-design-assets),
after the merges and the i18n commit, so it serves the merged code. It is still running.

| Command | Result |
|---|---|
| `npm run lint` | passed (app, shared, admin; Prettier clean) |
| `npm run test:ci` | passed: app 527, shared 53, admin 145 tests |
| `npm run test:tools` | passed: 43/43 |
| `LP_ENGINE_PREBUILT=1 npm run build` | passed (app + admin, no budget warning) |
| `npm run i18n:check` | passed |
| `npm run i18n:check-bundle` | passed ("no catalogue in 52 scripts; 3 files served") |
| `LP_ENGINE_PREBUILT=1 npm run test:engine` | passed: 16/16 |
| `node design-system/scripts/check.mjs` | passed (64 contrast pairs, 146 preview keys, 25 assets) |
| `cmp CLAUDE.md AGENTS.md` | identical |
| `npm run e2e` | **FAILED: 3 failed, 3 passed** |
| `npm run e2e:visual` | passed: 5/5; captures in `frontend/e2e/visual/output/` |
| `npm run e2e:hosted` | **not run**: port conflict (below) |

## 5. Failures, grouped by disjoint file zone

### Zone A — export format table (lot 7): `frontend/projects/app/src/app/export/export-format-table.html`

- Command: `npm run e2e --prefix frontend`
- Failing tests (all 3 have this one cause):
  - `e2e/editor/accessibility.spec.ts:35` › accessibility › the export dialog
  - `e2e/editor/journey.spec.ts:17` › exports: five formats with their sizes, then WASM
  - `e2e/editor/keyboard.spec.ts:77` › exports: five formats with their sizes, then WASM
- Excerpt:
  ```
  Locator: getByRole('table', { name: 'Every export format, with its size' })…getByRole('cell').first()
  Expected pattern: /^\d[\d.,]*\s?[a-zA-Z]+$/
  Received string:  " 15.9 kB "      (and " 16 kB " in the other two)
    at EditorPage.expectEveryFormatSized (e2e/editor/editor-page.ts:177)
  ```
- Cause: the `ready` case puts `{{ rawSize(row) }}` / `{{ gzipSize(row) }}` inside
  `@switch/@case` and `@if` blocks, on its own line, so the `<td class="size">` text keeps leading
  and trailing spaces. The e2e regex is anchored and does not trim them.
- Likely fix, for the corrections agent to choose: remove the whitespace in the template (for example
  wrap the size in a `<span>`, or put the interpolation on the same line as its tags), or trim in
  `frontend/e2e/editor/editor-page.ts:177-178`. The template fix keeps the e2e contract as it is.

## 6. Checks against criteria 2, 3 and 4 (merged zones)

- **C2** `rg "^\s*(button|input)\s*\{" frontend/projects/app/src`: no match in files wave A touched.
  Five matches remain, none of them touched by wave A (0 commits since `8db9f11`). They belong to
  later lots: `tools/tool-bar.scss:13`, `tools/import/import-image-button.ts:28`,
  `tools/import/import-sprite-sheet-button.ts:30` (lot 4), `palette/palette-entry-form.scss:52` and
  `timeline/tags/tag-dialog.scss:53` (lot 5).
- **C3** Hex colours in `frontend/projects/app/src/**/*.scss`: **none**. Emoji in templates:
  `timeline/layers/layer-list.html:25,47` (`👁`, `✕`, both `aria-hidden`), not touched by wave A
  (lot 5).
- **C4** `<main` in `frontend/projects/app/src/app/**`: only a mention in `ui/README.md`, no tag.
  Every routed page scrolls to the bottom at 390×640 (`measurements.json`: sign-in, sign-up,
  reset-password, verify-email, library→sign-in, settings, legal/terms, 404).
  **Gap, not caught by the rg:** two `main` landmarks are nested on C6 pages.
  `app.html:7` gives `ion-router-outlet` `role="main"`, and Ionic also sets `role="main"` on each
  `ion-content`. Seen in the browser on `/sign-in` and `/settings`: `ION-ROUTER-OUTLET#main-content` and
  `ION-CONTENT` are both `main`. This goes against C6 ("pas de `<main>` imbriqué") and lot 8b reported
  it. Zone: the shell and page template (`frontend/projects/app/src/app/app.html`, the `lp-page`
  template in `ui/`). Chained Playwright locators (`getByRole('main').getBy…`) still resolve.
- **C5 (extra)**: `measurements.json` shows the stage at 405 px (52.7 %) at both 1366×768 and
  1024×768, with `artworkFits: true`.
- `::ng-deep`: none in `frontend/projects`.

## 7. Browser review (server 4260, 1440×900, plus the visual captures)

Seen live: home/welcome (fr, light), the new-animation dialog, the editor after creating an animation,
export (fr, light), settings (fr, light, then dark and en applied instantly without a reload), the
editor and export in dark/en, and the 404 (fr, light). The 404 in dark/en and the editor at 390 px
(fr, dark) were checked on the `e2e:visual` captures. No page errors. The only console error is
`401 /api/v1/auth/session`, which is expected for a visitor.

Remarks, not failures:
- Export is enabled with no document open. The dialog then shows "Could not export" on every row.
  This was already so before wave A (`export-button.ts` has never had `disabled`). Zone
  `export/export-button.ts` (lot 3 or 7), a UX choice to confirm.
- The tool rail at 1440×900 cuts off the last buttons. The rail scrolls (`overflow-y: auto`), and
  lots 4 and 6 restyle it.
- At 390 px the canvas stage sits below the fold, after the document bar and the tool rail
  (lot 6, narrow screens).
- Theme and language reset on a full reload. This is expected: `WebPreferencesStore` keeps nothing
  in the browser (D37).
- The palette, layers and frise still use the old unstyled look (lots 4 and 5, not merged yet).
- An `input type=file` next to each import button is exposed as a second button with the same name,
  "Importer une image" (lot 4, not touched by wave A).

## 8. Not run, with the cause

- `npm run e2e:hosted`: global setup stops at
  `Error: Ports 8460, 8461, 8463, 8464 are in use: stop what listens there first.`
  (`e2e/hosted/stack/network.ts:26`). `docker compose up -d --wait` also starts the `server`
  (8460/8461) and `admin` (8463/8464) containers from `compose.yaml` + `compose.override.yaml`. They
  have been up for 6 h, and the suite wants to start its own binaries on those same ports. The method
  forbids stopping the shared services, so nothing was worked around (same as at milestone M1).
  Hosted specs not run as a result: including the quota fix, the library "Actions for …" menu (8a), the
  account and support page objects (8b), and the admin sign-in (9).
- Browser drawing on the canvas: `browser_run_code_unsafe` was refused by the session policy. Drawing
  is covered by `e2e` (the drawing steps of the journey passed) and `e2e:visual`.
