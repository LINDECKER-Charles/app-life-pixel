# Shared interface primitives

What every screen builds on, after [Rose Atelier](../../../../../../design-system/README.md)
(plan C3–C6). Read `design-system/docs/components.md` and `patterns.md` for the contracts; this
page says where each one lives and how to use it.

## Global classes

`frontend/projects/shared/src/styles/_components.scss` loads them once for the whole app, so
component stylesheets stay small (8 kB budget) and never restyle `button {}` or `input {}`.
Each partial of `styles/components/` documents its markup at its top.

| Classes                                                                                                                   | Contract                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lp-button` + `--primary`, `--secondary`, `--quiet`, `--danger`, `--compact`                                              | Native `<button>` for an action, `<a>` for navigation. One primary per decision area. `aria-pressed` makes any of them a toggle; `aria-busy="true"` a pending one; `disabled` dims it. |
| `lp-icon-button` (+ `lp-button--compact`)                                                                                 | Square, icon only: always an `aria-label`, and a tooltip where the lots add one.                                                                                                       |
| `lp-field`, `__label`, `__hint`, `__control`, `__unit`, `__error`; `lp-input` (+ `--compact`); `lp-check`; `lp-fieldset`  | A persistent label; hint and error tied with `aria-describedby`; `aria-invalid="true"` draws the error edge. The unit sits beside the value.                                           |
| `lp-pill` + `--success`, `--warning`, `--danger`, `--info`, `--accent`; `lp-pill__dot`                                    | A static state with a dot or an icon, never a button.                                                                                                                                  |
| `lp-banner` + `--success`, `--warning`, `--danger`, `--info`; `__icon`, `__body`, `__actions`                             | Use `lp-status-banner` rather than the classes alone.                                                                                                                                  |
| `lp-card` (`__title`), `lp-panel` (`--soft`), `lp-stack`, `lp-cluster`                                                    | Surfaces and spacing of content groups.                                                                                                                                                |
| `lp-modal` (`--wide`) on `ion-modal`; `lp-dialog`, `__header`, `__title`, `__context`, `__body`, `__failure`, `__actions` | Dialog anatomy: the body scrolls, the failure area and the action row stay visible; primary action first in reading order.                                                             |
| `lp-page` (`--reading`), `lp-page-header` (`__text`, `__title`, `__lead`, `__actions`), `lp-section`                      | Page template C6: `ion-content` > `.lp-page`, one `h1`, no `<main>` (the outlet is the main landmark).                                                                                 |
| `lp-tabs`, `lp-tab`, `lp-tab-panel`                                                                                       | Styles of the `tablist` pattern; its owner implements roving focus, arrows and Home/End.                                                                                               |
| `lp-menu`, `__header`, `__list`, `__item`, `__separator`                                                                  | Drawn by `lp-menu-button`.                                                                                                                                                             |
| `lp-visually-hidden` (`_accessibility.scss`)                                                                              | Text for assistive technology only.                                                                                                                                                    |

## Components

| Component                             | Contract                                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lp-icon` (`icon/`)                   | `name`: an `IconName` of `icon-paths.ts`; `size`: `small` 16 px, `medium` 20 px (default), `large` 24 px. Always `aria-hidden`; the control or the text beside it carries the meaning. Draws in `currentColor`.                                                                                                                                       |
| `lp-empty-state` (`empty-state/`)     | `heading` (required, translated), `description`, `headingLevel` (2 or 3), `hasPip`; the action is projected content. Pip is decorative (`alt=""`) and only for the welcome, the library and a project.                                                                                                                                                |
| `lp-status-banner` (`status-banner/`) | `variant`: `success`, `warning`, `danger`, `info`. Icon, the severity in words for screen readers, projected text; actions in an element marked `lpBannerActions`. `danger` is `role="alert"`, the others `role="status"`. No timer.                                                                                                                  |
| `lp-menu-button` (`menu/`)            | APG menu button: `label` names the menu, `triggerClass` styles the button. Project the trigger's content with `lpMenuTrigger`, an optional `lpMenuHeader`, then `role="menuitem"` links and buttons with `lp-menu__item`. Enter, Space, arrows open it; arrows, Home, End move; Escape closes and returns the focus; Tab and outside clicks close it. |

## Icons

`pencil`, `eraser`, `fill`, `line`, `rectangle`, `rectangle-filled`, `select`, `undo`, `redo`,
`import-image`, `import-sheet`, `help`, `zoom-in`, `zoom-out`, `fit`, `grid`, `onion-skin`, `eye`,
`eye-off`, `trash`, `plus`, `duplicate`, `edit`, `play`, `pause`, `stop`, `save`, `export`,
`download`, `copy`, `check`, `close`, `chevron-down`, `chevron-up`, `chevron-left`,
`chevron-right`, `menu`, `warning`, `error`, `info`, `success`, `layer`, `image`, `frame`, `tag`,
`settings`, `account`, `library`, `search`, `sign-out`.

A new icon goes into `ICON_PATHS` on the same 24-unit grid, outline first, with a solid part only
for a filled state. Never an emoji as an icon.
