# Accessibility and inclusive interaction

Target: WCAG 2.2 AA across the implemented application. This proposal defines acceptance criteria;
it does not certify the existing V1 or replace manual assistive-technology testing. The token
contrast pairs in [foundations](foundations.md) have been calculated; application behaviour must
be verified as components are adopted.

## Visual acceptance criteria

Normal text reaches 4.5:1 contrast, large text 3:1, and essential control boundaries and graphical
state indicators 3:1 against adjacent colours. Large text means at least 24 CSS px regular or
about 18.66 CSS px bold. Colour alone never identifies errors, selection or status. These are
the relevant [WCAG 2.2 contrast and colour criteria](https://www.w3.org/WAI/WCAG22/quickref/).

The application additionally chooses 44 px minimum touch targets as a product standard. Fine
pointer desktop controls may use 36 px targets. Both exceed the 24 CSS px minimum target-size
criterion without relying on its spacing exceptions. Inline text links follow normal text flow.
Buttons that look small may have a larger hit area only if it does not overlap adjacent targets.

Use `control-border` for necessary input boundaries and `border` only for decoration. Keep text
at full opacity. Disabled controls remain readable and include a nearby reason when the cause is
not apparent. A selected swatch has an outline and selection marker in addition to its colour.
Status chips include words and a distinct icon, such as a check, warning triangle or error symbol.

Focus uses a 3 px `focus` outline with a 3 px offset on approved neutral or soft surfaces. The
offset separates it from the coloured button fill. It stays visible beneath sticky headers,
sheets and notification overlays. Never remove an outline without providing an equivalent.
Test rings against their actual adjacent pixels; token contrast alone is insufficient on artwork.

At 200% text size and 400% page zoom, controls retain readable labels and usable order. At 320 CSS
px width, surrounding forms and navigation reflow; spatial drawing areas may use bounded panning.
Avoid fixed-height text containers. Respect user text-spacing changes without clipping labels,
help or error copy. Test long French strings with real catalogues before release.

## Keyboard and focus contracts

Use native HTML elements before ARIA. A link changes location; a button performs an action.
Every interactive element has an accessible name containing its visible label. Prefer labels
connected with `for` and `id`; a placeholder is never the only label. An icon-only tool has a
translated accessible name and a tooltip available on hover and focus.

| Pattern | Required behaviour |
| --- | --- |
| Page shell | Skip link to main content; logical heading order; named navigation regions |
| Button or link | Tab reaches it; Enter activates; Space activates buttons |
| Tool group | Named toolbar; documented arrow navigation if roving focus is implemented |
| Toggle tool | Name plus `aria-pressed`; state independent of focus |
| Tab set | Named tabs, selected state, matching panel; arrow-key movement |
| Dialog | Name, initial focus, contained focus, Escape close, return to trigger |
| Menu | Keyboard open, arrow navigation, Escape close; focus returns to trigger |
| Select or number field | Native keyboard interaction; constraints and units described |
| Frame or layer list | Active item announced; explicit move controls alongside dragging |
| Tooltip | Dismissible with Escape; no interactive content; no essential help only on hover |

Implement custom patterns according to the
[WAI-ARIA Authoring Practices patterns](https://www.w3.org/WAI/ARIA/apg/patterns/).
Do not add menu or grid roles to simple lists unless their complete keyboard model is supported.
Use actual links in route navigation, not tab roles. Do not apply `role="application"` to the
entire editor.

When a dialog closes, restore focus to its invoker or a logical surviving target after deletion.
For irreversible deletion, initial focus goes to the safe option. Closing an export dialog must
not cancel completed work. Background controls are inert while a modal is open. A popover within a
dialog belongs to that dialog's focus scope.

Single-character shortcuts are active only in the editor's relevant focus context, can be disabled
or remapped, and never fire in a text field or during input-method composition. Publish shortcuts
in translated help. Display the appropriate Control or Command modifier for the platform.
Escape leaves a temporary mode before leaving the entire screen. Avoid overriding browser zoom.

## Canvas, palette and timeline

The canvas needs a focused editing alternative, not only an image description. Name the editing
region and explain its available keyboard controls before the user enters it. Show the current
coordinates, tool, selected colour, frame and layer in ordinary readable UI.

For keyboard editing, move a virtual cursor with arrow keys and trigger the selected tool with an
explicit action. Multi-point operations need start, endpoint and cancel controls. Provide numeric
coordinate entry where precise placement is required. Send these actions to the existing engine;
do not reimplement pixel operations in TypeScript. If V1 lacks these controls, track that gap before
claiming keyboard conformance for drawing.

The palette exposes each swatch's index and hexadecimal value, with a transparency description
where relevant. User colour names supplement the value. Do not demand that user artwork itself
matches the application's contrast palette; distinguish document content from interface chrome.
Provide a high-contrast cursor outline or neutral backing so the pointer remains visible over
arbitrary art. The grid and selection overlay must be distinguishable at high zoom.

Frames and layers have names and positions announced by assistive technology. A frame additionally
exposes its duration and selected state. Dragging is optional: move earlier/later and up/down
controls produce the same operations and retain focus. Scrolling a timeline does not change its
selected frame. Playback never automatically steals keyboard focus.

## Forms, validation and authentication

Every field has a visible label; format guidance precedes submission. Required status appears in
text or a consistently explained marker. Associate help and error text with `aria-describedby`,
and set `aria-invalid` only when the field is invalid. Preserve entered values after an error.
On unsuccessful submit, focus an error summary linking to invalid fields, or the single invalid
field. Server error codes are translated through the catalogues; do not display raw codes.

Allow password managers, autofill and paste. Use appropriate autocomplete tokens and a labelled
password reveal button. Avoid a memory test as the only authentication path. Instructions explain
whether work can be saved before the user reaches a save action. Guest, local and hosted contexts
use distinct copy instead of a vague universal saved status.

## Announcements and asynchronous work

Use a polite status region for completed saves, imports and exports. Errors requiring immediate
attention may use an alert, once per event. Do not announce every pointer movement, pixel change,
playback frame, repeated retry or progress percentage. Coalesce relevant updates and let users
request detailed coordinate feedback.

Busy controls retain their label, expose the busy state and prevent duplicate submission. Long
operations expose progress when it is meaningful and cancellation only when supported. Focus
stays where the user put it. A toast supplements persistent feedback; an export download, error
recovery or undo action remains reachable after the toast disappears.

Loading placeholders reserve space, stay out of the accessibility tree and stop shimmering under
reduced motion. Empty results explain how to clear filters; a first-use empty state provides a
clear next step. An offline or failed save state remains visible until resolution.

## Motion, appearance and sensory comfort

Use `prefers-reduced-motion` for every decorative effect, independent of the application's theme.
The duration tokens become zero, but consumers must remove independent keyframes and smooth
scrolling too. No decorative animation autoplays indefinitely. Animation previews have explicit
play/pause controls; switching theme never changes document playback or export data.

Do not add flashing celebrations or strobing selection indicators. User-authored animation can
contain unexpected flashing; preview does not autoplay on file selection, gallery navigation or
initial load. A user action starts playback. Keep pause immediately reachable and preserve it
while the preview loads. A content-safety review may add further playback controls later.

Respect forced colours and browser zoom. The token stylesheet supplies system-colour overrides;
components still need native borders and semantic markup so controls survive forced colours.
Avoid `forced-color-adjust: none` except for the actual colour artwork or palette content, and
keep their surrounding labels and selection markers adaptable. Test light, dark and system themes.

Decorative mascots and icons have empty alternative text or `aria-hidden="true"`. A meaningful
image receives a concise description of its purpose. Icons inside labelled buttons do not repeat
the button's name. No essential instruction is baked into an illustration.

## Review matrix and release evidence

Run this matrix on the implemented component or journey, not only on the design-system specimen.
Record the browser, operating system, viewport, input method, result and any remaining defect.
Automated checks are useful evidence but cannot establish conformance on their own.

| Review | Required evidence |
| --- | --- |
| Contrast | Scripted token pairs plus real rendered hover, focus, error and selected states |
| Keyboard | Create, import, edit, save and export without a pointer; no trap or lost focus |
| Screen reader | NVDA on Windows plus VoiceOver or TalkBack for the target mobile surface |
| Zoom and reflow | 200% text, 400% browser zoom, 320 px width, long French labels |
| Pointer and touch | 44 px coarse targets, no overlap, drag alternatives, no hover dependency |
| Appearance | Light, dark, OS automatic, forced colours and reduced motion |
| Recovery | Failed save/import/export, offline recovery, destructive cancellation |
| Semantics | Named controls, headings, landmarks, field associations, status announcements |

Use the repository's accessibility scripts and Playwright coverage as implementation progresses.
Log failures with a reproduction and the affected journey. Do not mark a criterion passed based
on the existence of a token, ARIA attribute, screenshot or design rule alone.
