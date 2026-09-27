# Wave B — verification of the reported failures

Branch `feat/rose-atelier-redesign`, verified commit `88e801bdfb909b6218e73708af3b4173292db58a`
(2026-09-27). `git status --porcelain` at the start and the end of this session shows only the
untracked `docs/plans/` (reports, not committed): every tracked file is committed, no drift.

## Services

- Dev server 4260: down at the start (no listener, same as `wb-failures.md`'s note that a prior
  agent had to restart it — it does not survive between sessions). Restarted identically with
  `LP_ENGINE_PREBUILT=1 npm start --prefix frontend`; it answers 200 and serves the code at
  `88e801b` (bundle timestamp `2026-09-27T02:52:46Z`). Left running for the next agent.
- Local stack: all 5 containers already `Up … (healthy)` (`postgres`, `objectstore`, `mail`,
  `server`, `admin`), not restarted (already running).

## Battery (serial, root, same commands as `wb-failures.md`)

| Command | Result | Log |
|---|---|---|
| `npm run lint` | pass | `wb-verify-lint.log` |
| `npm run test:ci` | **FAIL**: app 1 failed/564 passed+1; shared 53 passed; admin 145 passed | `wb-verify-testci.log`, `-shared.log`, `-admin.log` |
| `npm run test:tools` | pass (43) | `wb-verify-tools.log` |
| `LP_ENGINE_PREBUILT=1 npm run build` | pass, no budget warning | `wb-verify-build.log` |
| `npm run i18n:check` | pass (2 catalogues) | `wb-verify-i18n.log` |
| `npm run i18n:check-bundle` | pass | `wb-verify-i18nb.log` |
| `LP_ENGINE_PREBUILT=1 npm run test:engine` | pass (16) | `wb-verify-engine.log` |
| `node design-system/scripts/check.mjs` | pass | `wb-verify-ds.log` |
| `cmp CLAUDE.md AGENTS.md` | identical | — |
| `LP_ENGINE_PREBUILT=1 npm run e2e` | **1 failed, 5 passed** (was 2 failed, 4 passed) | `wb-verify-e2e.log` |
| `LP_ENGINE_PREBUILT=1 npm run e2e:visual` | pass (5) | `wb-verify-visual.log`, `wb-verify-measurements.json` |
| `npm run e2e:hosted` | **not run** (environment, unchanged) | — |

## Zone A — canvas view bar: fixed, confirmed

`e2e/editor/journey.spec.ts:17` now passes (`ok 1 … (6.1s)`). The commit `88e801b` reserves
`14ch` for the position field (`view-bar.scss`) and adds a spec asserting the reading stays in the
same field and the status line keeps the same parts across pointer enter/leave
(`view-bar.spec.ts`). `keyboard.spec.ts:77` (criterion 6's keyboard path) also passes. Manual
check not repeated in-browser this session; the e2e pass is direct evidence the artwork no longer
jumps under the pointer.

## Zone B — shortcuts dialog: scrollable-region fixed, one violation remains

`e2e/editor/accessibility.spec.ts:42` still **fails**, but the originally reported violation
(`scrollable-region-focusable`) is gone: `.lp-dialog__body` is now `role="region" tabindex="0"
aria-labelledby="shortcuts-help-title"` (commit `2610f3e`), confirmed by browser inspection at
1280×720 (region takes focus, list scrolls with arrow keys).

A different, **new** violation now surfaces in the same axe run:

```
color-contrast (serious): Elements must meet minimum color contrast ratio thresholds —
.row:nth-child(1) > kbd, .row:nth-child(2) > kbd, .row:nth-child(9) > kbd,
.row:nth-child(10) > kbd, .row:nth-child(11) > kbd, .row:nth-child(12) > kbd,
.row:nth-child(13) > kbd
```

Investigated live in the browser (French, `/editor`, shortcuts dialog open): every flagged `kbd`
computes to the exact same `color: rgb(115, 91, 102)` on `background-color: rgb(255, 237, 241)`
as every passing one (`--lp-color-text-muted` on `--lp-color-surface-soft`), a **5.45:1** ratio
(WCAG relative-luminance formula), well above the 4.5:1 threshold for 14px text. The flagged rows
are exactly the ones whose key-cap text has more than one character (`Ctrl/⌘+S`, `Ctrl/⌘+E`,
`Ctrl/⌘+Z`, `Ctrl/⌘+Shift+Z`, `Ctrl/⌘+Y`, `Shift+R`, `Shift+G`); every single-character key cap
(`B`, `E`, `G`, `L`, `R`, `M`, `?`, `+`, `-`, `0`, `[`, `]`, `P`, `,`, `.`, `O`) passes. Since the
computed colours are identical across both groups, this reads as an axe-core sampling artefact
tied to multi-character/mixed-glyph runs in the monospace `kbd` font (the `⌘` glyph in particular
falls back to another font), not a real contrast deficiency — but this session does not fix
anything, so it is reported as-is. Files: `tools/shortcuts-help-dialog.ts` (`.keys` rule),
`shared/src/styles/_tokens.scss` (`--lp-color-text-muted`, `--lp-color-surface-soft`).

## New failure outside zones A/B: `save-dialog.spec.ts` (test:ci)

`ng test app` fails one test in `frontend/projects/app/src/app/library/save/save-dialog.spec.ts`
(around line 184–193, the exact line shifts slightly between runs): `seriousViolations(root)`
returns a `non-empty-title`/aria-label violation on Ionic's own `#ion-overlay-N .modal-wrapper`
(`<div role="dialog" aria-modal="true" tabindex="-1" class="modal-wrapper ion-overlay-wrapper">`).
Reproduced twice, including with `ng test app --include "**/save-dialog.spec.ts"` in isolation
(same failure, different overlay id), so it is deterministic, not order-dependent flake from
other spec files. This file is untouched by the lot 4/5/5 fixes or the wave-B integration commits
(`git log` shows its last touch is `ac44d4c`/`c66652e`/`c5d8c9c`, the flow-dialogs lot, merged
before `wb-failures.md` was written) and `wb-failures.md` reported `test:ci` fully green at
`a3081a7`. This is a new regression or a newly-flaky test surfaced between `a3081a7` and `88e801b`
(the only tracked change on that path is none — no commit touches `library/save/` after
`a3081a7`), pointing to environment/library-version sensitivity (Ionic's overlay id counter,
jsdom) rather than to wave-B code. Not investigated further: outside this session's file zones
and the mission forbids fixing.

## Not run

- `npm run e2e:hosted`: same cause as `wb-failures.md` — ports 8460, 8461, 8463, 8464 still held
  by the stack's `server` and `admin` containers (checked live: all four open). The method
  forbids stopping shared services, so this stays `not_run`.
- No commit hooks to replay (`.git/hooks` holds only samples, no `core.hooksPath`); no commit was
  made this session (verification only).

## Plan §5 criteria re-checked

- **2** pass (unchanged): `rg "^\s*(button|input)\s*\{"` over `frontend/projects/app/src` gives
  no match.
- **3** pass (unchanged): no hex colour in the app's `.scss` outside the named
  `CHECKER_LIGHT`/`CHECKER_DARK`/`GRID_COLOR` constants in `canvas-renderer.ts` (a `.ts` file, not
  matched by the `.scss` grep anyway).
- **5** pass (unchanged), from `wb-verify-measurements.json`: 1366×768 stage 405 px (0.527),
  surface 950×358; 1024×768 stage 405 px (0.527), surface 608×328; artwork fits both times.
- **6** now fully pass for the reviewed gap: zone A's e2e (`journey.spec.ts`) passes, so pointer
  drawing under the view bar is reliable again; `keyboard.spec.ts` still passes. No other gap
  reported in `wb-failures.md` for this criterion.
- **7** pass by the unit tests: `save-state.spec.ts` and `save-state-pill.spec.ts` (distinct from
  the failing `save-dialog.spec.ts`) are among the passing tests in this run.

## Summary of remaining failures

1. Shortcuts dialog `color-contrast` axe violation (likely a false positive, see above) —
   `e2e/editor/accessibility.spec.ts:42`.
2. `save-dialog.spec.ts` Ionic overlay `non-empty-title`/aria violation — `ng test app`,
   unrelated to zones A/B, first seen in this session.
3. `npm run e2e:hosted` — still blocked by the stack's own container ports (environment,
   unchanged from `wb-failures.md`).
