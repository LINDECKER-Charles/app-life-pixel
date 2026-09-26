# Journeys and screen structure

This document specifies the proposed experience for the existing V1 product. It does not change
routes, implement features, or grant a new product capability. The companion preview uses fixtures.
**Existing** describes the audited implementation. **Design target** describes work to implement.

## People and outcomes

| Person | Outcome | First useful result |
|---|---|---|
| Curious visitor | Try pixel animation before creating an account | Play two drawn frames |
| Pixel artist | Draw, organise and deliver animations | Save a project and download an export |
| App or game developer | Add a small controllable animation | Download WASM and copy a snippet |
| Agent-assisted developer | Connect their own agent to the same library | Preview an MCP edit |
| Desktop creator | Work locally without a connection or account | Reopen a local animation |
| Administrator | Resolve a service or support issue safely | Find an issue and its next action |

The proposed interface feels like a welcoming creative studio. It still uses precise tool names,
clear status, and predictable controls. Pip, the pixel chibi bunny, supports welcome, empty and
celebration moments; it never explains a security, deletion, quota, or data-loss warning.

## Information architecture

| Existing route | Existing purpose | Design target |
|---|---|---|
| `/` | Redirects to `/editor` | Keep direct entry; optional welcome inside editor |
| `/editor` | New or in-memory work | Create, draw, animate, save, export |
| `/editor/:animationId` | Open a library animation | Same workspace, with source and save status |
| `/library` | Projects and searchable animations | Visual collection with clear empty state |
| `/library/projects/:projectId` | Project animations | Breadcrumb, title, scoped contents |
| `/settings` | Language, theme, motion, desktop folder | Group preferences by purpose |
| `/settings/agents` | Desktop agent setup | Copyable local connection instructions |
| `/settings/tokens` | Signed-in hosted token management | Explicit scopes, expiry and revocation |
| `/sign-in`, `/sign-up` | Email/password sign-in | Short forms preserving return context |
| `/verify-email` | Email verification | Result with next action; never block editing |
| `/reset-password` | Request reset | Clear, non-disclosing submission result |
| `/reset-password/confirm` | Set a replacement password | Validation, success, return to sign-in |
| `/account` | Usage, language, data export, deletion | Account details and clear security actions |
| `/support`, `/support/:requestId` | Support requests | Submission and conversation status |
| `/legal/:page` | Terms, privacy, legal notice | Readable article with language support |
| Unknown route | Not-found page | Back to editor; preserve in-memory work |

Hosted library access needs an account. Desktop library access does not. Keep Library discoverable
to visitors, explain saving requirements when selected, and return to the intended destination.
Keep platform-specific links out of unsupported contexts. Do not show a locked billing destination.

Global navigation: Editor, Library, Settings, Help. Keep account actions in a labelled menu on web.
Desktop uses its local identity and library location rather than a sign-in prompt. Existing desktop
Help currently encounters a hosted-only support route; its replacement needs an explicit UX task.

## 1. First use: a useful result before registration

**Existing:** the root opens the editor; New opens the size/title form. No onboarding route exists.

**Design target:** show a dismissible welcome in the empty workspace. Its primary action is
Create animation; its secondary action is Import PNG. A short sentence explains that drawing and
exporting need no account, and that visitors must save to an account or export before leaving.
Pip appears once next to this text, outside the working canvas.

1. Create animation opens the existing form: name, width and height, using limits from the engine.
2. Place focus in the name field. Explain dimensions as pixels and preserve entered values on error.
3. Submit opens the workspace with Pencil selected and the initial frame visible.
4. Show one dismissible hint: choose a colour, then draw. Give access to keyboard help.
5. After the first stroke, expose the next action: duplicate the frame, alter it, then play.

Do not force a tour, ask for interests, invent templates, or request registration before drawing.
Preset sizes and guided hints are design proposals; presets must be validated by the engine.
No guest work or guest preferences may be persisted in browser storage under the current contract.

## 2. Draw, animate, preview

**Existing:** tools, palette, layers, frames, duration, tags, playback, undo/redo and shortcuts.

Desktop wireframe, read from top to bottom:

```text
App navigation                     Library · Settings · Help · Account
Animation title · save status      New · Save · Export
Tools     | neutral canvas viewport                 | indexed palette
          | zoom / grid / drawing position          | colour controls
Layers    | tag ranges + frame strip + durations    | playback preview
```

The centre remains the visual priority. Use warm cream and pale pink for surrounding surfaces;
keep the canvas stage, checkerboard, pixel grid and exported preview neutral. Never tint artwork.
Use integer pixel rendering for artwork; soft rounded containers belong to the surrounding UI.

1. Select a tool: show its pressed state, accessible name and shortcut. Keep undo/redo nearby.
2. Select a palette colour: show selection and its index without relying only on colour.
3. Draw on the active frame and layer. Identify both near the canvas or timeline.
4. Duplicate or add a frame. Keep the current frame visible when the strip scrolls.
5. Change duration in milliseconds; show units and engine-provided constraints next to the field.
6. Toggle onion skin; distinguish editor-only guidance from the actual export preview.
7. Play and pause explicitly. Preview uses the shipping player, so it remains the reference.
8. Add a named frame range if needed; choose loop or play once.

Do not add layer opacity, blend modes, ping-pong, a state machine or palette variants: V1 has none.
Palette index 0 is transparent. Give transparency an explicit checkerboard and text description.
Keyboard alternatives must remain available for canvas actions and frame/layer reordering.

## 3. Import

**Existing:** PNG image and sprite-sheet import, validation, palette reduction and slicing exist.

1. Choose Import PNG or Import sprite sheet with distinct labels.
2. Show selected filename, dimensions and intended operation before confirmation where available.
3. For sheets, group cell size and slicing controls and explain their units.
4. Describe palette reduction before import; a replacement must disclose what it replaces.
5. Keep validation errors next to affected inputs; retain the rest of the form and current artwork.
6. On success, focus the editor and announce imported content with its resulting frame count.

A visual slicing preview is a design proposal. Never imply Aseprite or arbitrary image support.
Enforce image limits through the existing engine; documentation examples are not new limits.

## 4. Save and return

**Existing:** saving is explicit. Visitors can sign in/up and resume saving on return to the editor.
The flow chooses or creates a project, writes the document, then opens its saved animation route.

1. A visitor chooses Save. Explain that an account saves work to their library; retain the document.
2. Offer Create free account, Sign in, and Keep editing. Return to the same editor after success.
3. For a first save, choose a project or create one inline; ask for only the necessary information.
4. Show Saving while the write runs. Announce Saved only after confirmed success.
5. Any later edit changes the visible state to Unsaved changes. Do not claim autosave.
6. Returning creators can open, search, rename, move, duplicate or delete existing work.

In-page authentication preserves the in-memory document; a full refresh still loses guest work.
The browser owns its unload warning wording. A design cannot promise recovery after tab closure.
Email verification has a dedicated status but does not block drawing, saving or exporting in V1.

Library wireframe: heading and Create animation action; project list; labelled animation search;
results; explicit load-more control. Thumbnail tiles are a design proposal requiring preview data.
Retain a compact list option for long names and efficient browsing. No fabricated activity feed.

## 5. Save failure and conflict recovery

| Event | Keep available | Explain and offer |
|---|---|---|
| Connection/write failure | Current artwork, editing and export | Work is still open; retry save |
| Session expired | Current artwork | Sign in again and resume the requested save |
| Newer stored version | Current artwork | Copy, reload saved version, overwrite |
| Animation deleted elsewhere | Current artwork as unsaved | Save as a new animation |
| Storage quota exceeded | Read, export and delete | Open library to free storage, then retry |
| Local folder unavailable | In-memory work and export | Reconnect or choose another folder |

Conflict copy should be the recommended safe action. Explain that reload discards current edits
and overwrite replaces the stored version. Never preselect a destructive choice.
Show real usage and limit; if either is unavailable, show an unavailable state, never a fake zero.
Do not offer Upgrade in V1: paid plans are future scope. Nothing is automatically deleted at quota.

## 6. Export and integrate

**Existing:** all five formats, actual size comparison and framework snippets are available free.

Dialog wireframe: title and close control; export options; format/size table; download actions;
integration snippet with framework selector and copy action. Keep it scrollable on short screens.

1. Open Export from the editor. Describe the current animation and relevant selected range.
2. Prepare WASM, GIF, APNG, sprite sheet plus JSON, and PNG frames.
3. Show actual raw and gzip byte sizes as they arrive; mark the smallest comparable result.
4. Explain the consequence of format choice: controllable player, animation file, or image assets.
5. Download the chosen output; avoid success wording until the export is ready.
6. For integration, select HTML, Angular, React or Vue, then copy the generated snippet.
7. Explain where the downloaded files belong and that snippet paths must match the user's project.
8. Offer Return to editing. A small optional Pip moment can accompany successful export.

Never invent measured sizes in production. Label all preview values as examples. Do not suggest
that downloading publishes work, creates a live embed, or hosts a URL. Those capabilities are absent
from V1.

## 7. Settings, account and agents

Settings groups Language, Appearance, Motion and, on desktop, Library folder and Agent setup.
Theme supports light, dark and system. Motion follows system or a reduced preference. Changing
language updates the interface without reload. Desktop settings stay available offline.

Account groups address/verification, storage, language and data export. Keep sign-out separate
from permanent account deletion. Deletion requires the existing identity/password confirmation
and explicit consequences; remove decoration from this flow.

Hosted MCP setup leads to token management: label, scopes, expiry, one-time secret display, copy,
and revocation. Never put a token in a URL, analytics event or screenshot fixture. Local desktop
setup explains the bundled CLI, library location and copied configuration without a hosted token.
OAuth connectors are future scope. Life Pixel does not run or pay for the user's AI model.

## 8. Support and administration

Customer support needs a subject, category, message, submission status and request reference.
After submission, show the request thread and next expected step without promising an unverified
response time. A failed submission retains the message. Legal pages remain plain, readable text.

The separate admin app keeps Overview, Monitoring, Logs, Alerts, Users, Support and Audit.
Existing routes are `/`, `/monitoring/:panel`, `/logs`, `/alerts`, `/users`, `/users/:id`,
`/support`, `/support/:id`, `/audit`, plus `/sign-in` and the not-found page.
Show environment and data freshness prominently. Use dense tables with explicit filters, sortable
headers, pagination and labelled actions. Preserve useful timestamps and identifiers.
Re-use accessible controls and tokens; reduce decorative pink and omit Pip in operational incidents.
Account suspension and other destructive actions need impact text and confirmation.

## Distribution and viewport adaptations

| Surface | Layout and behaviour |
|---|---|
| Wide browser/desktop | Persistent panels; canvas grows; timeline remains reachable |
| Small laptop | Compact labelled tool rail; collapsible palette; preserve canvas area |
| Narrow browser | One active auxiliary panel; reachable drawing tools and explicit panel tabs |
| Touch/tablet | Large targets, no hover dependence, explicit undo, reorder alternatives |
| Desktop offline | Local library/save/export/MCP remain usable; no hosted sign-in requirement |
| Hosted offline | Keep current in-memory editing/export when loaded; saving reports failure |
| Self-hosted | Same capability model from configuration; no assumed hosted domain or billing |
| Android | Future M6; reuse responsive patterns without claiming an available app |

The current editor stacks its regions at 767px. A tabbed or bottom-sheet narrow layout is a design
target requiring implementation and keyboard testing. Keep canvas pan distinct from page scrolling.
Reflow forms and chrome at 320 CSS pixels and 200% text zoom; the two-dimensional canvas and
timeline may scroll inside labelled regions. Opening a keyboard must not cover the active field.

Offline is not the same as saved. Guest work is never durable. Do not promise a service worker,
background sync, cached documents, automatic conflict merging, or restored browser sessions.

## Journey acceptance checks

- A first-time visitor can identify Create, draw two frames, play and export without registering.
- A returning user can identify the open animation, active layer/frame and truthful save state.
- Save after authentication returns to the original in-memory work; cancellation loses nothing.
- Import and save failures preserve work; conflict choices explain exactly what each replaces.
- A full quota still permits export and deletion; the interface offers no unavailable billing flow.
- Desktop users can complete their local journey offline without hosted-only controls.
- The full path works by keyboard, with visible focus, reduced motion, and English/French labels.
- Administrative status is understandable without relying on colour or illustration.

## Evidence and implementation references

- [Product scope](../../docs/product.md), [V1 scope](../../docs/v1/README.md).
- [Decisions](../../docs/decisions.md): D24, D25, D29, D33, D37 are important boundaries.
- [Application routes](../../frontend/projects/app/src/app/app.routes.ts).
- [Editor layout](../../frontend/projects/app/src/app/editor/editor-page.html).
- [Save flow](../../frontend/projects/app/src/app/library/save/save-flow.ts).
- [Save states](../../frontend/projects/app/src/app/library/save/save-dialog.html).
- [Export dialog](../../frontend/projects/app/src/app/export/export-dialog.ts).
- [Admin routes](../../frontend/projects/admin/src/app/app.routes.ts).
