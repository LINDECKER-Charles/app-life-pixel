# Lot 1: visual foundations (report)

Branch `feat/rose-atelier-redesign`. Commits:
- 60159ac: asset copy
- b94aa98: tokens, Ionic, focus, `_base.scss`, editor.md
- 741a082: canvas
- 9de485e: parity test

## What changed

- `_tokens.scss` holds the Rose Atelier light, dark and forced-colour values under the existing
  names. No token was removed or renamed. `--lp-color-border` is now the DS `control-border`, and
  the DS decorative `border` becomes `--lp-color-border-subtle`.
- New tokens:
  - colours: surface-soft/raised; accent hover/soft/text; success, warning and info, each with
    `-bg`; danger-bg; disabled; scrim (+ `-rgb`, `--lp-scrim-opacity`); rose, peach, mint and
    lavender; canvas-bg; checker-light/-dark
  - shadows: small, medium and large
  - z-index scale: see decisions
  - durations and easings
  - `--lp-control-height`, `-compact` and `-large`
  - focus width and offset
  - type steps
  - radius-xlarge and radius-2xlarge
  - layout widths
- Values kept as they were: spacing 1–6 (no step added) and `--lp-font-size-large` at 1.25rem
  (used by dialog titles; the new `--lp-font-size-lead` is 1.125rem). `--lp-radius-large` moves
  from 12 px to 14 px (DS fields and buttons). `--lp-font-weight-bold` moves from 600 to 700
  (`--lp-font-weight-semibold` is 600).
- Ionic: overlay background = raised surface, `ion-modal --background` = raised surface, the
  backdrop uses the DS scrim, and primary shade = accent-hover. Existing dialogs still paint their
  own `.dialog` on the background; that is lot 7's job.
- Focus is 3 px at a 3 px offset. Reduced motion also zeroes `--lp-duration-*`.
- `_base.scss` (app only, through `_index`):
  - local `@font-face` Nunito (`/design-system/fonts/nunito-variable.ttf`, `font-display: swap`)
  - `--lp-font-family` = Nunito, Trebuchet MS, then the complete system stack
  - body type, headings at 700 / 1.25, monospace code, links in accent-text
- Admin: loads only tokens and accessibility, so it keeps `--lp-font-family-system`.
  `admin/src/styles.scss` is unchanged.
- The CSP `font-src 'self'` (server.md) already allows the local font.
- Canvas: checkerboard `#ffffff` / `#ececee`, grid `#68686f` (DS `--lp-canvas-grid`, checked in
  tokens.css), surround `--lp-color-canvas-bg`.
- Outside the listed files (needed to keep things working):
  - `.dockerignore` and `docker/app.Dockerfile` copy `design-system/assets/`. Otherwise
    `build:app` would fail in the app image (e2e:hosted, release).
  - `docs/v1/editor.md` said the focus outline was 2 px.

## Measured contrast (WCAG relative luminance)

| Pair | Light | Dark |
|---|---|---|
| text / background | 13.42 | 15.55 |
| text / surface | 14.10 | 13.78 (DS) |
| text-muted / background | 5.85 | 9.12 |
| text-muted / surface | 6.15 | 8.08 (DS) |
| control border / surface | 3.64 | 4.65 |
| control border / background | 3.46 | 5.25 |
| on-accent / accent | 6.19 | 7.88 |
| accent / background | 5.89 | 8.81 |
| link (accent-text) / background | 7.91 | 10.83 |
| focus / background | 5.56 | 8.66 |
| on-danger / danger (production only; DS has no role) | 6.98 (white) | 9.19 (`#35212c`) |
| grid / checker light, dark, black artwork | 5.53, 4.69, 3.80 | same (theme-independent) |
| text / canvas-bg | 10.51 | 15.12 |

`node design-system/scripts/check.mjs` passes 64 DS pairs. The grid can disappear on artwork near
`#68686f`, as any single-colour grid would. On black it stays readable (3.80:1).

## Admin

- Started with `npx ng serve admin --port 4263`, with no stack running. It redirects to
  `/sign-in`, and the only console error is the 401 on `/api/admin/v1/auth/session`.
- Captures `captures/lot1-admin-{light,dark}-1440.png`: system stack, rose button, readable
  fields in both themes. Server stopped afterwards.
- `--lpa-chart-*` against the new backgrounds:
  - light, on `#fff8f5`: 7.53 / 4.78 / 5.22 / 5.75
  - dark, on `#241b23`: 8.39 / 10.02 / 8.70 / 9.22
  - dark, on surface `#30242e`: at least 7.43
  - All are ≥ 3:1, so there is no change. Charts were not seen rendered: they sit behind sign-in.

## Screenshots

- `npm run e2e:visual`: 5/5 passed, 80 PNGs in `frontend/e2e/visual/output/`.
- The lot 0 baseline was saved first in `captures/lot0-baseline/`.
- Canvas stage: 370 px = 0.481 of 768 at 1366 and at 1024 (baseline 374 px = 0.487). The small
  loss comes from Nunito's metrics and line-height 1.6. Criterion §5.5 still fails; that is
  lot 3/4's job.
- Browser at `localhost:4260/editor`, 1440 px, light and dark: Nunito, rose accents, neutral
  checkerboard and grid in both themes, violet 3 px focus ring on the canvas. At 390 px the layout
  problems are the same as the baseline (lot 6).
- Grey UA-default timeline buttons in dark are identical in the baseline (lot 5).

## Checks

- Passed:
  - `lint`
  - `test:ci`: app 424, shared 53, admin 145
  - `test:tools`: 43, including 6 for the copy and 9 for parity. A mutation check (one wrong
    value, one unmapped colour) makes 2 fail.
  - `build`: component budgets OK
  - `i18n:check`
  - DS `check.mjs`
  - `cmp CLAUDE.md AGENTS.md`
  - `e2e`: 6/6, axe green
  - `e2e:visual`: 5/5
- `build` warning: the initial bundle is 937.9 kB against a 750 kB warning (1 MB error). The whole
  global stylesheet is 27.9 kB, so the overshoot predates this lot.
- 4260 server: it was down (its log shows no error; the process ended with the previous session).
  I restarted it with `LP_ENGINE_PREBUILT=1 npm start --prefix frontend`, which copies the assets.
  It is left running (log `dev-server-4260.log`).
