# Interaction patterns and state contracts

These are design specifications for future application integration. The isolated preview only
demonstrates them. Existing state handling must be preserved while presentation changes.

## Shared interaction rules

- One primary action per decision area; use a verb and a concrete object where space permits.
- Put the recommended action first in reading order without moving keyboard focus automatically.
- Keep undo, save, export and close predictable across editor states.
- Destructive actions use explicit language and a danger treatment distinct from raspberry actions.
- Labels remain visible; a placeholder is an example, never the field's only name.
- A selected item uses shape, text or an icon as well as colour.
- Every pointer action has a keyboard path; drag-and-drop has a non-drag alternative.
- Keep user artwork, search input and form input on recoverable failures.
- Pink surfaces carry atmosphere; strong text, boundaries and focus carry interaction meaning.

## Cross-surface state matrix

| State | Presentation | Required action or recovery |
|---|---|---|
| Initial loading | Stable structure, short status text, `aria-busy` on affected region | Wait |
| Partial loading | Render ready items; local progress | Continue with available work |
| Empty collection | Heading, reason, one next action; optional Pip | Create or import |
| No search results | Repeat the query safely; neutral message | Clear or change search |
| Ready | Relevant content and truthful metadata | Main task |
| Pending mutation | Button progress label; prevent duplicate submission | Keep context visible |
| Success | Result visible; brief polite announcement | Continue or return |
| Field error | Text next to field and invalid state | Correct the identified input |
| Recoverable failure | Plain explanation and retained work | Retry or choose an alternative |
| Access required | Explain the capability needing identity | Sign in or keep editing |
| Access denied | Explain inability; reveal no private details | Return to a safe view |
| Offline/unavailable | Identify affected service or local folder | Reconnect, retry or export |
| Conflict | Explain both current and saved versions | Copy, reload or overwrite |
| Destructive confirmation | Name the item and consequence | Cancel or explicitly confirm |
| Unknown/not found | No private metadata; explain absence | Library or editor |

Do not replace a loaded page with a global spinner during a local action. A loading skeleton must
match the eventual structure and never pulse under reduced motion. Do not show an empty state until
loading has completed successfully. An error is not an empty collection.

## Empty states

Use a short title, a useful explanation and a specific action. Keep the state inside the affected
region so the surrounding navigation remains available. Decorative Pip is hidden from assistive
technology when the adjacent text communicates the same meaning.

| Location | State-specific message | Primary action |
|---|---|---|
| New visitor editor | Explain what can be made and the guest save limitation | Create animation |
| Library | No saved projects yet | Create animation |
| Project | No animations in this project | Create animation |
| Search | No matching animation, while other saved work still exists | Clear search |
| Hosted tokens | No tokens created | Create token |
| Support | No requests yet | Contact support |
| Admin table | No records match the current filters | Clear filters |

Only welcome/library/project empties may use Pip. Search and administrative empties stay compact.
Do not place a full-page illustration inside a small timeline, palette or table cell.

## Buttons, links and tool controls

Navigation uses a link; mutation uses a button. Primary actions use raspberry fill with a verified
contrasting label. Secondary actions use a surface and an identifiable border. Quiet controls still
have visible hover and keyboard focus states. Hover is never the only way to discover an action.

Required states: default, hover, focus-visible, pressed, disabled and pending. Toggle controls also
have selected/unselected states and `aria-pressed` or the appropriate native role. Distinguish the
currently selected drawing tool from the temporary pointer-down state.

Use a 44px minimum touch target; a compact visible icon may sit inside that larger hit area.
Dense desktop exceptions must still meet the accessibility acceptance criteria and provide spacing.
Icon-only controls need a translated accessible name and a visible tooltip on hover and focus.
Tooltips include shortcuts where present; Escape closes them; they must not obscure the target.

Disable an impossible action only when its reason is evident or explained nearby. Keep explanatory
text readable rather than dimming the whole panel. Pending buttons retain their width and label.

## Forms and validation

1. Give the form a title describing the outcome, such as Create animation or Import sprite sheet.
2. Group related fields; put units and concise constraints next to the relevant input.
3. Validate on submit and after a previously invalid field changes; avoid first-keystroke errors.
4. On failure, focus a summary linking to invalid fields; retain valid values.
5. Pair `aria-invalid` with `aria-describedby`; announce submission errors once.
6. On success, return focus to the next useful location, not always the first form control.

Use engine limits for canvas/frame/palette data and API validation for account/server data. The
design system does not introduce independent domain constants. Numeric fields accept input in the
active locale where supported and present unambiguous units. Parsing remains application code.

Passwords support paste, password managers and a labelled show/hide button. Never clear a complete
form because one field fails. Translate the server's stable error code into useful action text.

## Dialogs and sheets

A modal has a translated title, a visible close/cancel action and an initial focus strategy.
Trap focus while open, make the background inert, and restore focus to the trigger when dismissed.
Escape closes an ordinary dismissible dialog; pending irreversible operations explain any temporary
restriction. Pressing Enter submits only the active form, never a destructive default by accident.

Use an alert dialog only for a short urgent decision, such as confirmed data loss. Long export,
import and token forms use ordinary dialogs. At narrow widths, the same content becomes a sheet
or full-width dialog with reachable actions and safe-area padding. Do not stack modal flows.

Short screens scroll dialog content while keeping the title and available dismissal understandable.
Opening the software keyboard must leave the active field and its error visible. Focus must not be
hidden behind a sticky action area. Outside-click dismissal never silently discards a dirty form.

## Saving and unsaved work

Show the animation title and one explicit save state near Save:

| State | Label intent | Behaviour |
|---|---|---|
| New guest work | Not saved | Save opens account explanation; export remains available |
| Changed stored work | Unsaved changes | Save writes current work |
| Write in progress | Saving… | Suppress duplicate saves; keep editor state visible |
| Confirmed write | Saved | Announce once; next edit removes the claim |
| Write failed | Could not save | Retain work, explain reason, enable recovery |

Saving is explicit in V1. Do not display autosave, synchronised, backed up or offline-ready claims.
The browser unload guard provides the browser's own warning. New/Open/Reload actions that replace
current work must use the existing discard confirmation. Export is never proof of a saved document.

For a conflict, describe that another version was saved. Recommend Save a copy. Reload saved version
must disclose loss of current edits. Overwrite saved version must disclose replacing stored work.
Closing the conflict leaves the current work open. A failed recovery does not close the dialog.

## Destructive actions

Place Delete away from Open and Duplicate. A confirmation names the exact animation/project/account
and states whether its contents are included. Show Cancel as an available safe action; focus it
initially when the consequence is irreversible. Avoid playful copy, Pip, confetti and reward sounds.

After success, remove the item, announce the result and move focus to the nearest remaining item
or collection heading. Do not offer Undo unless a real restoration capability exists.
Account deletion preserves the implemented password and email confirmation requirements.
Token revocation identifies the token and explains that connected agents using it lose access.

## Search, collections and tables

A labelled search field is paired with an explicit submit action where existing behaviour requires
it. A clear action restores the original list. Searching preserves the query during requests and
errors. Do not promise instant results if the implementation searches on submit.

Each animation shows its title and useful dimensions/frame count. Proposed preview thumbnails use
nearest-neighbour artwork; they have neutral backgrounds and never crop important edge pixels.
An overflow action menu may replace a long action row after keyboard and touch behaviour is tested.
Use real cursor pagination; retain the explicit Load more action rather than inventing total pages.

Use native tables for admin/export comparisons, with captions and associated column headers.
Sorting exposes direction. Dense tables may scroll horizontally inside a labelled region; keep
actions reachable by keyboard. Loading or unavailable metrics never become zero-valued metrics.

## Editor-specific patterns

| Region | Required behaviour |
|---|---|
| Tool rail | Label, shortcut, selection, keyboard activation and focus |
| Canvas | Neutral stage, visible keyboard cursor, clear active frame/layer, pan/zoom help |
| Palette | Index, selected colour, transparency, meaningful accessible colour description |
| Layers | Name, visibility, selected state, reorder alternative and deletion consequence |
| Frames | Thumbnail, number, selection, duration with unit, reorder alternative |
| Tags | Name, range and loop/once; distinguish range from frame selection |
| Preview | Explicit Play/Pause; show loading/error independently of editor availability |

Do not animate the canvas surface as decoration. Preserve exact artwork colours, including during
hover, selection and disabled states. Frame/layer removal uses the application's existing undo
where supported; it must not imply that deleting an entire saved animation is similarly reversible.

A pointer gesture must not unexpectedly trigger both canvas drawing and page scrolling. Provide
explicit controls for touch users. Roving focus, if introduced, documents arrow keys and exit keys
and is tested with a screen reader; never trap focus inside the canvas application region.

## Notifications and connection status

Use inline feedback for problems tied to a form or object. Use a polite status message for a
completed save/copy/export. Reserve an assertive alert for a blocking failure requiring action.
Never announce every pointer movement or animation frame. Debounce coordinate/status announcements.

Toasts must not hold the only copy of an error, consequence, token or recovery action. Do not make
users race a timer to act. Keep essential actions inline until resolved or explicitly dismissed.
Optional success decoration is brief, silent, non-blocking and removed under reduced motion.

Hosted connection loss affects remote saving and library requests. Desktop local operation remains
independent of the hosted service. A local filesystem failure identifies a folder problem rather
than calling it a network outage. Never promise queued background uploads or automatic retry.

## Account, access and privacy

Visitors can edit/export; a hosted account enables library persistence. State this benefit at the
save gate. A failed sign-in keeps the document and the intended return destination in memory.
Sign-out must warn before any action that would discard current unsaved work.

Token secrets are shown once, copied through an explicit action, and never sent to logging or
telemetry. A copy failure offers manual selection. Preview examples use obvious non-secret fixture
text and cannot authenticate. Scope and expiry explanations appear before token creation.

Respect distribution configuration: local desktop users do not encounter web subscription controls;
self-hosted instances do not inherit hosted marketing or infrastructure assumptions. V1 desktop
sends no telemetry at all. The design system does not add a consent or analytics implementation.

## Motion and responsive behaviour

Motion explains state transitions and never delays input. Respect both operating-system and saved
reduced-motion preference. Remove decorative loops, bounce, shake and parallax. User-started
artwork playback is a separate creative action; provide Pause and do not autoplay sample art.

Navigation, forms and dialogs reflow before labels truncate. Allow two-line labels in French.
At narrow widths, keep the main creative action reachable and show one auxiliary editor panel at
a time as a proposed layout. Preserve position/selection when changing panel. Do not rely on hover.

## Accessibility verification for every pattern

- Text contrast: at least 4.5:1 for normal text, 3:1 for qualifying large text.
- Required control boundaries, selected indicators and meaningful graphics: at least 3:1.
- Verify actual foreground/background pairs, including hover, selected, dark theme and focus.
- Every action has an accessible name; headings and landmarks follow a meaningful order.
- Keyboard focus is visible, not obscured, and restored after overlays and mutations.
- Colour, position, sound and mascot emotion never carry the only meaning.
- At 200% text zoom and narrow reflow, no action or essential text disappears.
- Reduced motion removes ornamental movement; the artwork player remains explicitly controllable.
- Screen readers receive useful state changes without repetitive drawing/playback announcements.
- English and French fit with realistic long titles, error text and OS font fallback.

These requirements are acceptance criteria, not a claim that the proposed preview or application
has completed a WCAG audit. Test integrated controls with keyboard and assistive tools.

## Source contracts

- [Existing save prompts](../../frontend/projects/app/src/app/library/save/save-dialog.html).
- [Unsaved work guard](../../frontend/projects/app/src/app/shell/unsaved-work-guard.ts).
- [Shared accessibility styles](../../frontend/projects/shared/src/styles/_accessibility.scss).
- [Current settings](../../frontend/projects/app/src/app/settings/settings-page.html).
- [Internationalisation rules](../../docs/i18n.md).
