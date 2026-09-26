# Component contracts

Status: design proposal. These contracts describe the intended interface; they do not claim that
new Angular components or journeys have shipped. Existing feature components remain the starting
point. Every example uses the same warm cream, pale pink and raspberry system.

## Shared anatomy and states

Every control has a purpose, an accessible name, a visible focus state and a predictable result.
Decoration never supplies the only label. Pip, the chibi bunny, appears only in welcome, empty and
success moments; it stays outside dense controls, errors, account deletion and admin operations.

| State | Visual treatment | Behavior and accessibility |
| --- | --- | --- |
| Default | Neutral surface; strong ink; clear boundary | Name states the action or value |
| Hover | Modest surface or border change | Same information remains available without hover |
| Focus | Distinct outline with separation from the edge | Never replace with only a shadow |
| Pressed | Slightly deeper surface | Feedback lasts only while pointer/key is down |
| Selected | Raspberry marker and shape/check/text | `aria-pressed`, `checked` or active route |
| Disabled | Muted but legible | Native `disabled`; explain relevant prerequisites |
| Pending | Stable width, busy indicator and verb | Prevent repeat requests; announce completion |
| Invalid | Error icon, text and strong boundary | `aria-invalid`; connect message to field |
| Read-only | Legible value on quiet surface | Allow focus, selection and copying where useful |

Use soft borders to separate decorative panels. Use strong borders for input boundaries, selected
states and controls whose shape must be perceived. A pale pink divider is not a control boundary.
Raspberry `#a93663` is the light theme action color. Palette values belong in tokens.

Comfortable controls target at least 44 × 44 CSS px. Compact editor controls may use smaller visual
shapes inside an adequately spaced hit area. Never rely on a 16 px glyph as the whole touch target.
Text wraps or enlarges its container at zoom; compact layouts must not truncate essential labels.

## Buttons, links and icon buttons

Anatomy: optional leading icon, action label, optional trailing affordance, optional busy indicator.
Use a native button for an action and an anchor for navigation or download. Do not nest controls.

| Variant | Intended use | Treatment |
| --- | --- | --- |
| Primary | Main next step for a section or dialog | Raspberry fill, contrast-tested foreground |
| Secondary | Supporting action | Light surface, strong border and dark label |
| Quiet | Repeated low-priority action | Transparent surface, explicit hover/focus feedback |
| Destructive | Confirm deletion or overwrite | Danger color and specific destructive verb |
| Toggle | Tool, visibility or playback setting | Selected marker and `aria-pressed` |
| Icon | Familiar repeated action in dense tools | Accessible name and tooltip, adequate hit area |

One dominant primary action per local decision. “Save a copy” and “Overwrite” must have visibly
different consequence levels. Do not turn every timeline command into a primary button.
Keep loading labels meaningful, such as “Saving…”, and announce the eventual result once.
Enter and Space activate native buttons; ordinary links retain browser open-in-new-tab behavior.
Tooltips appear on keyboard focus and hover and never contain required instructions.

## Fields and forms

Anatomy: persistent label, optional short explanation, control, optional suffix/unit, helper text
and validation message. Placeholder text is an example, never the only label.

- Text: visible label, explicit autocomplete, appropriate input type and allowed length.
- Password: existing `lp-password-field` provides the starting point; retain autocomplete and
  an accessible show/hide action whose name reflects its current function.
- Numeric: show the unit beside the value; prefer `inputmode="numeric"` for integer values.
  Limits come from the engine/API. Width, height and frame duration are not new UI constants.
- Select: prefer native select for short simple lists; retain Ionic select where mobile platform
  behavior is useful. The label remains visible after selection.
- Checkbox: independent choices, visible checked mark, label expands its click target.
- Radio group: mutually exclusive choices in `fieldset` and `legend`; native arrow behavior.
- Switch: an immediate binary preference with a clear on/off state; use checkbox semantics unless
  a tested switch pattern is needed. Do not disguise a submit action as a switch.
- Textarea: multiline instructions above the field; visible count only when it helps a real limit.

Do not show errors before an untouched field has been submitted or meaningfully edited. Preserve
entered values after failure. On submit, focus the first invalid field or a linked error summary.
Associate hints and errors with `aria-describedby`; avoid announcing the same error twice.
Asynchronous validation has a pending state and does not erase an existing valid value.
Authentication errors keep the current server disclosure policy; copy must not invent account facts.

## Dialogs, sheets and confirmations

Anatomy: named title, concise context, content, inline failure area and action row. Long content
scrolls inside a bounded viewport. The close control and action row remain reachable at zoom.

Use Ionic modal in the app and native `dialog.showModal()` in the admin where already established.
Both must trap focus, make background content inert, support Escape when dismissal is allowed,
restore focus to the trigger and provide an accessible title. Test Ionic's shadow DOM label.
The current `lpModalLabel` directive is relevant to existing save dialogs.

- Entry dialogs: focus the first editable field after opening.
- Destructive confirmations: initially focus the least destructive action; name the target.
- Pending mutation: prevent duplicate confirmation. Define cancellation only if it can actually
  cancel the operation; closing a spinner does not imply that a server request was cancelled.
- Dirty form: ask before discarding meaningful input, using the existing unsaved-work flow.
- Small screens: use a full-width sheet or full-screen form with the same fields and semantics.
- Nested decisions: replace the dialog content or close the first before opening another.

New animation, tag, palette, import, save-conflict and export dialogs share this anatomy, but keep
their feature-owned validation. Do not build a universal form component with arbitrary mode flags.

## Toasts, banners, badges and empty states

| Pattern | Content | Persistence and semantics |
| --- | --- | --- |
| Success toast | Completed action and optional link | Polite status; nonessential may expire |
| Error notification | What failed and a recovery action | Remains until read/dismissed; no timer |
| Warning banner | Current risk and next step | Persistent near the relevant content |
| Environment banner | Staging/production context | Persistent explicit text in admin |
| Badge or pill | Compact state or category | Label plus icon/shape where state matters |
| Empty state | Why empty and useful next step | Differentiate no content, no result and failure |
| Skeleton | Approximate pending structure | Decorative; parent owns `aria-busy` |

Toast messages must not obscure the canvas tool in use or the bottom action row. Use `role="status"`
for routine completion and `role="alert"` only for urgent, newly introduced problems. Existing
engine notifications are persistent error messages; do not convert them into disappearing praise.

Static status pills are not buttons. A removable filter pill has a separately named remove button.
Do not show “Saved” until the persistence operation succeeds. “Unsaved changes”, “Saving”, “Saved”
and “Save failed” are distinct, written states. Worker recovery is a separate concern.
Pip may illustrate an empty library, but never replace the explanation and creation action.

## Navigation, tabs and settings

App shell: wordmark, current area, primary navigation and account entry. Keep the editor identity
and document title separate. Current route uses `aria-current="page"`; responsive menus retain
labels and keyboard access. A mobile bottom navigation is a proposal, not an existing component.

Route changes remain links. Use tabs only to swap peer panels in the same context: one active tab,
`tablist`/`tab`/`tabpanel`, connected IDs and roving tabindex. Arrow keys move focus; Home/End move
to the first/last tab. Activate on focus only when the panel appears without latency.

Settings group language, appearance, motion and platform-specific storage/integration preferences.
Radio cards may style native inputs for system/light/dark and system/reduced motion. The selected
card has a check plus text. Changing theme must affect every open overlay and both native/Ionic UI.
Keep hosted account/token settings and desktop library/agent settings behind current capabilities.

## Editor toolbar and canvas

Toolbar anatomy: drawing tools, history actions, imports, shortcuts help. Existing tools are
pencil, eraser, fill, line, rectangle and selection; filled rectangle is a separate toggle.
Preserve shortcuts B, E, G, L, R and M and the existing history/help registrations.
Use one consistent icon style. Tool buttons keep translated accessible names, visible selected
markers and tooltip shortcut hints. Do not invent eyedropper or brush-size controls. Preserve the
existing onion-skin state and O shortcut; a more visible toggle is a presentation improvement.

The canvas is the visual anchor. Keep its drawing area neutral so UI pink does not bias the artist's
color judgment. Checkerboard, transparency and pixel colors remain distinct from the brand palette.
Keep crisp nearest-neighbor rendering, pointer capture and viewport transforms under existing
canvas/engine services. No cute overlays, decorative grids or shadows inside the artwork.

The existing keyboard pixel cursor uses arrows and Enter; preserve its focus outline and polite
feedback. Tool shortcuts must ignore text inputs and open overlays. Zoom/pan must remain operable
without a wheel or drag-only gesture. Content around the canvas can reflow; pixel coordinates
cannot.

## Palette, layers and timeline

Palette anatomy: selected color indicator, swatch grid, add action, edit/remove actions and limit
feedback. Entry zero is transparency/eraser. Use a double contrasting selection ring or check so
selection remains visible on arbitrary user colors. Actual colors never become theme tokens.
Keep edit/remove accessible without hover. Reordering must have keyboard alternatives to dragging.
Color editing includes an explicit value and alpha channel; invalid input preserves the last value.

Layer row: visibility toggle, editable name, reorder affordance and delete action. Keep rename
confirmation on Enter/blur and reversion on Escape. Hidden is written/announced as hidden, not only
faded. If active-layer styling is added, it must reflect an actual engine selection capability.

Timeline anatomy: frame commands, tags, horizontally scrollable frame strip, durations and preview.
Each frame shows a thumbnail, a visible number, duration and selected marker. Preserve multi-select
behavior, keyboard range selection and reorder commands. Scroll focused frames into view.
No animation is needed to indicate frame selection. Names and durations remain readable at zoom.

Tags show their name, span and playback intent. Overlaps must not obscure selection. Tag editing
retains validated frame bounds and looping/play-once options. A colored tag still needs text.
Playback is an explicit play/stop toggle with a stable preview region; never start it on hover.
Reduced motion suppresses UI flourish; an artist-requested playback remains a deliberate action.

## Library and project items

Library anatomy: page title, creation entry, projects, animation search, result collection and
cursor-based “Load more”. Current rows can evolve into cards when thumbnail data is available.
Do not imply that thumbnails, sort modes or recent-file metadata already exist in the API.

Project item: name link, animation count, rename/duplicate/delete menu. Animation item: title link,
dimensions/frame count, rename/move/duplicate/delete menu. The whole card must not be a button
containing other buttons. Use a primary title link and separate actions.
Show loading, loaded-empty, filtered-empty, loading-more and load failure separately.
Deletion confirmation names the item and consequence. Failed deletion leaves it visible.
Search retains the query on navigation or retry only when implemented by the actual state owner.

## Export

Anatomy: title, target tag, scale, format comparison, download actions and integration snippets.
V1 offers WASM, GIF, APNG, sprite sheet and PNG frames. Keep computed sizes visible with units and
label the smallest result in words. Do not suggest estimated sizes are final before compilation.
Format rows show preparing, ready, failed and downloading states independently when appropriate.

For newcomers, explanatory text can describe a format's use; the format choice must never obscure
WASM's integration options. Copy actions report copied/failed states, and code remains selectable.
Use neutral monospace surfaces for snippets. Do not recolor pixel previews to match the page.
An export success may show Pip outside the preview; always retain a clear next action.

## Admin tables, filters and charts

Admin uses the same typography, semantic colors and controls with restrained decoration and compact
spacing. Keep environment, permission and destructive-action context more prominent than branding.
Retain native table headers, captions or accessible names, row links and explicit action labels.
Sorting uses a button inside its header plus `aria-sort` on the active column. Do not make every row
a single giant click target. Horizontal overflow preserves all data and remains keyboard reachable.

Filters retain labels, applied values, time context and a clear reset action. Empty, unavailable,
not-configured, forbidden, loading and failed data sources are different states. Never show zero
when a source is unavailable. Keep audit rows and support deadlines legible without extra pills.

Chart anatomy: title, unit, time range, current/previous legend, plot and accessible summary/data.
Use distinct accessible series colors and line styles. Existing previous-period series are dashed;
retain that distinction. Avoid near-identical pinks for all series. Missing data remains gaps.
Chart line/axis contrast and focus are tested on both themes; subtle gridlines are decorative.
Any new hover detail must have a keyboard-accessible equivalent. A graph cannot be the only way to
read operational values. Keep redraw behavior consistent when theme or size changes.

## Definition of ready for a component

- Specify owner, purpose, anatomy, variants, states and platform applicability.
- Document keyboard behavior, accessible name, announcements and focus destination.
- Check long English/French strings, empty values, pending/error states and reduced motion.
- Verify light/dark themes and forced colors; separate control border from decorative separator.
- Exercise 320 px narrow layouts, touch, 200% zoom and the relevant editor workspace widths.
- Test behavior through existing feature services; do not duplicate engine or API domain rules.
- Add a preview example and a focused regression test when behavior changes.
