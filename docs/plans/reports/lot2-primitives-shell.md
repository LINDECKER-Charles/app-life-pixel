# Lot 2 – primitives and shell (C3–C6)

Commits on `feat/rose-atelier-redesign`: dea17dc (shared), 5215da2 (i18n), 8db9f11 (app).

## Delivered
- `_components.scss` forwards 7 partials (`styles/components/`): buttons, fields, feedback, surfaces, dialog, page, navigation.
- `ui/`: `lp-icon` (50 icons, `icon-paths.ts`), `lp-empty-state`, `lp-status-banner`, `lp-menu-button` (APG menu button), README.
- Shell: skip link to the routed page, native header (mark + name, nav, Help hidden on desktop), account menu on `lp-menu-button`, footer, in-flow engine notifications (danger-bg, icon + text, `--lp-z-toast`).
- i18n: `common.dismiss`, `common.status.{error,info,success,warning}`, `shell.account_menu.signed_in_as`, `shell.notifications.label`, `shell.skip_to_content`.

## Checks
- lint OK; test:ci app 453 / shared 53 / admin 145 passed; build OK (app initial 965 kB, budget warning pre-existing at 750 kB, below the 1 MB error); i18n:check OK; design-system check OK; e2e 6/6.
- Browser 4260: 1440/390, light/dark, fr/en. Only console error: the known 401 on `/api/v1/auth/session`.

## Notes
- `tabindex="-1"` on `ion-router-outlet` (a shadow host) removed the whole page from Tab order in Chrome (e2e keyboard failure). The skip link now focuses the routed page element instead.
- The signed-in account menu cannot be shown in the browser without the hosted API; it is covered by unit tests.
- The `git mv` of the header (shell/ → shell/header/) was already in the index and went into the shared commit as pure renames; the app commit carries their content. dea17dc alone does not build the app (app.ts still imports the old path); 8db9f11 does. No rewrite (rebase/reset are excluded).
- `browser_run_code_unsafe` was refused by the session policy; the skip link capture was not taken, the behaviour is covered by app.spec.ts and e2e.
- Server 4260 was down at start and was restarted identically (`LP_ENGINE_PREBUILT=1 npm start --prefix frontend`); it is still running.
