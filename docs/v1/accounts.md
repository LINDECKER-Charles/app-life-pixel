# Accounts and library — H5, H6, H7, H8, H16

Sign-in with an email address and a password (D29), the library behind an account, and the legal
pages. Visitors keep using the editor without an account; nothing of theirs is saved (D37).

## H5 — Accounts and sessions

### Data

A migration completes H4's `accounts` and adds two tables. H5 also updates H4's test helper that
creates accounts, since it adds required columns.

```sql
alter table accounts
  add column email citext not null unique,
  add column password_hash text not null,
  add column email_verified_at timestamptz,
  add column language text not null default 'en',
  add column status text not null default 'active' check (status in ('active', 'suspended'));

create table sessions (
  token_hash bytea primary key,         -- SHA-256 of the cookie's token
  account_id uuid not null references accounts (id) on delete cascade,
  created_at timestamptz not null,
  last_seen_at timestamptz not null,
  expires_at timestamptz not null
);
create index sessions_account on sessions (account_id);

create table email_tokens (
  token_hash bytea primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  purpose text not null check (purpose in ('verify_email', 'reset_password')),
  created_at timestamptz not null,
  expires_at timestamptz not null,
  used_at timestamptz
);
```

### Rules

- **Addresses**: trimmed; at most `EMAIL_MAX_CHARS`; one `@` with text on both sides and a dot in
  the domain, else `auth.email_invalid`. Case does not matter (`citext`).
- **Passwords**: `PASSWORD_MIN_CHARS` to `PASSWORD_MAX_CHARS` characters, no other rule; Argon2id,
  19,456 KiB, 2 passes, 1 lane, stored as a PHC string, on a blocking thread; rehashed at sign-in
  when the parameters change. An unknown address still spends one hash, so that timing says nothing.
- **Sessions**: 32 random bytes, base64url, in the cookie `__Host-lp_session` — `Secure`,
  `HttpOnly`, `SameSite=Lax`, `Path=/`, `Max-Age` 30 days —, stored as their SHA-256. A request
  more than a day after `last_seen_at` pushes the expiry 30 days further and resends the cookie.
  Expired sessions are purged daily.
- **CSRF**: the token is base64url(HMAC-SHA256(`LP_SESSION_SECRET`, session hash)), returned with
  the session. A `POST`, `PUT`, `PATCH` or `DELETE` carrying the session cookie must send it in
  `X-CSRF-Token` and come with an `Origin` equal to `LP_PUBLIC_URL`'s or listed in
  `LP_ALLOWED_ORIGINS`, else `auth.csrf`. Sign-up and sign-in, which have no session yet, check the
  origin only.
- **Tokens by email**: 32 random bytes, stored as their SHA-256, single use; 7 days to verify an
  address, one hour to reset a password.
- **Suspension**: a suspended account cannot sign in, and its sessions stop working
  (`auth.account_suspended`).
- Signing up says when an address is taken, under the `sign_up` rate limit; it signs in at once and
  sends the verification email. An unverified address blocks nothing in V1; the app shows a
  reminder with a resend button.

### Routes

| Route | Body | Answer | Errors |
|---|---|---|---|
| `POST /api/v1/auth/sign-up` | `{ email, password, language }` | `201` Session, cookie | `auth.email_invalid`, `auth.password_length`, `auth.email_taken`, `account.language` |
| `POST /api/v1/auth/sign-in` | `{ email, password }` | `200` Session, cookie | `auth.invalid_credentials`, `auth.account_suspended` |
| `POST /api/v1/auth/sign-out` | — | `204`, cookie cleared | — |
| `GET /api/v1/auth/session` | — | `200` Session | `auth.unauthenticated` |
| `POST /api/v1/auth/verify-email` | `{ token }` | `204` | `auth.token_invalid` |
| `POST /api/v1/auth/verify-email/resend` | — | `202` | — |
| `POST /api/v1/auth/password-reset` | `{ email }` | `202`, whether the address exists or not | — |
| `POST /api/v1/auth/password-reset/confirm` | `{ token, password }` | `204`; every session of the account ends | `auth.token_invalid`, `auth.password_length` |
| `PUT /api/v1/auth/password` | `{ currentPassword, newPassword }` | `204`; the other sessions end | `auth.current_password`, `auth.password_length` |

Session: `{ "account": Account, "csrfToken": "…" }`, Account as in H6. Rate limits: `sign_up`,
`sign_in`, `password_reset`, `verification_resend` ([server.md](server.md#rate-limits)).
`auth_events_total{event}` counts `sign_up`, `sign_in`, `sign_in_failed`, `sign_out`,
`password_reset`, `password_changed`.

### Emails

- `service::accounts` defines the ports `AccountStore`, `SessionStore`, `EmailTokenStore` and
  `Mailer`, and the use cases in `Accounts`; `server` implements the stores on Postgres and the
  mailer on SMTP (`lettre`, `LP_SMTP_URL`).
- `Mailer::send(Email { to, language, message })`, with `message` one of `VerifyEmail { link }`,
  `ResetPassword { link }`, `PasswordChanged`, `SupportReply { link }` (H9). The adapter renders
  `email.<message>.subject` and `email.<message>.body` from the catalogues in the account's
  language, English as fallback, replacing simple `{link}` arguments; it sends plain text and a
  minimal HTML part with the link.
- Links: `<LP_PUBLIC_URL>/verify-email?token=…`, `<LP_PUBLIC_URL>/reset-password/confirm?token=…`.
- `PasswordChanged` is sent after a reset and after a change.

### Codes

| Code | Params | Status |
|---|---|---|
| `auth.unauthenticated`, `auth.invalid_credentials` | — | 401 |
| `auth.email_invalid` | — | 422 |
| `auth.password_length` | `min`, `max` | 422 |
| `auth.email_taken` | — | 409 |
| `auth.token_invalid` | — | 400 |
| `auth.current_password`, `auth.account_suspended`, `auth.csrf` | — | 403 |
| `account.language` | `available` | 422 |

**Tests** (`stack-tests`, Mailpit): each route's success and each error; the cookie's attributes;
sliding expiry; CSRF refused without the token, with a wrong one, and from another origin; a reset
ending every session; a change ending the others; single-use and expired tokens; the emails in
English and French, with working links; timing-neutral sign-in for an unknown address; a suspended
account.

## H6 — Library and account API

### Types

```json
{ "Project": { "id": "…", "name": "…", "animationCount": 3, "createdAt": "…", "updatedAt": "…" },
  "Animation": { "id": "…", "projectId": "…", "title": "…", "width": 32, "height": 32,
                 "frameCount": 8, "documentBytes": 5120, "version": 4,
                 "createdAt": "…", "updatedAt": "…" },
  "Page": { "items": [], "nextCursor": null },
  "Account": { "id": "…", "email": "…", "emailVerified": false, "language": "fr", "plan": "free",
               "storage": { "usedBytes": 5120, "limitBytes": 100000000 }, "createdAt": "…" } }
```

### Routes

| Route | Body | Answer |
|---|---|---|
| `GET /api/v1/account` | — | `200` Account |
| `PATCH /api/v1/account` | `{ language }` | `200` Account |
| `GET /api/v1/account/export` | — | `200` zip |
| `DELETE /api/v1/account` | `{ password }` | `204`, cookie cleared |
| `GET /api/v1/projects` | `?cursor&limit` | `200` Page of Project |
| `POST /api/v1/projects` | `{ name }` | `201` Project |
| `GET`, `PATCH`, `DELETE /api/v1/projects/{id}` | `PATCH`: `{ name }` | `200`, `200`, `204` |
| `POST /api/v1/projects/{id}/duplicate` | `{ name }` | `201` Project |
| `POST /api/v1/projects/{id}/animations` | the document, `application/vnd.life-pixel.animation+json` | `201` Animation, `ETag` |
| `GET /api/v1/animations` | `?project&q&cursor&limit` | `200` Page of Animation |
| `GET /api/v1/animations/{id}` | — | `200` Animation |
| `PATCH /api/v1/animations/{id}` | `{ title?, projectId? }`; `If-Match` with a title | `200` Animation |
| `GET /api/v1/animations/{id}/document` | — | `200` the document, `ETag` |
| `PUT /api/v1/animations/{id}/document` | the document; `If-Match` required | `200` Animation, `ETag` |
| `POST /api/v1/animations/{id}/duplicate` | `{ title, projectId? }` | `201` Animation |
| `DELETE /api/v1/animations/{id}` | — | `204` |

- Every route needs a session (`auth.unauthenticated`). The `ETag` is the version, `"4"`; a `PUT`
  without `If-Match` is `document.version_required` (428), with a stale one
  `document.version_conflict` (412). Documents are limited to `MAX_DOCUMENT_BYTES` (413); the
  quota answers `quota.storage_exceeded` (409).
- **Data export**: a zip laid out like a [local library](service.md#h2--local-library) — the desktop
  app can open it once unzipped —, plus `account.json` (`email`, `language`, `plan`, `createdAt`);
  built in a temporary file on a blocking thread, streamed as
  `life-pixel-export-<YYYY-MM-DD>.zip`.
- **Deletion** asks the password again (`auth.current_password`), then deletes the documents, the
  account and — by cascade — its sessions, tokens, projects, animations and support requests; the
  sweeper collects any object left. Records `account_deleted`.
- `quota_rejections_total{kind="storage"}` counts refusals.

**Tests** (`stack-tests`): each route and error; owner isolation; the conflict and quota paths; a
data export unzipped and opened by H2's adapter; deletion leaving nothing of the account.

## H7 — Account screens

`frontend/projects/app/src/app/account/`:

| File | Content |
|---|---|
| `session-store.ts` | `SessionStore`: `account` as a signal, the CSRF token, `load()`, sign-in, sign-up, sign-out; gives `ApiClient` the CSRF header |
| `sign-in-page.ts`, `sign-up-page.ts` | the forms |
| `verify-email-page.ts` | reads `token` from the URL, verifies, says the result |
| `reset-password-page.ts`, `reset-password-confirm-page.ts` | request a link; set a new password |
| `account-page.ts` | the address and its verification (resend), storage used against the quota, language, data export, deletion, sign-out |
| `account-menu.ts` | the header menu: "Sign in", or the address with Account, Library, Sign out |

- `SessionStore.load()` runs at start-up on the hosted app — never on the desktop — through
  `GET /auth/session`. A `401` on a later call clears the session; a `426` opens a blocking dialog
  asking to reload.
- Forms: labelled Ionic inputs, `autocomplete` set (`email`, `current-password`, `new-password`),
  lengths from the engine's limits, the server's codes shown next to their field as
  `errors.<code>`, focus moved to the first error, a show-password toggle.
- Deleting the account asks for the password and an explicit confirmation naming the address.
- Sign-up links H16's terms and privacy policy (`/legal/terms`, `/legal/privacy`).
- Sign-in and sign-up are pages of the same app: the work in the editor survives them, and a
  `returnUrl` brings the person back.
- Saving a language calls `PATCH /account`; sizes are formatted with `Intl`.
- **Keys**: `account.`, `auth.`.

**Tests**: validation and the display of each code; the session restored at start; the CSRF
header on writes; sign-out; the `426` dialog; the return to the editor with the work intact; axe on
every page.

## H8 — Library and saving

`frontend/projects/app/src/app/library/`:

```ts
export interface LibraryStore {
  listProjects(page?: PageQuery): Promise<Page<Project>>;
  createProject(name: string): Promise<Project>;
  renameProject(id: string, name: string): Promise<Project>;
  duplicateProject(id: string, name: string): Promise<Project>;
  deleteProject(id: string): Promise<void>;
  listAnimations(filter: AnimationFilter, page?: PageQuery): Promise<Page<AnimationSummary>>;
  createAnimation(projectId: string, document: Uint8Array): Promise<AnimationSummary>;
  openDocument(id: string): Promise<{ summary: AnimationSummary; document: Uint8Array }>;
  saveDocument(id: string, document: Uint8Array, version: number): Promise<AnimationSummary>;
  renameAnimation(id: string, title: string, version: number): Promise<AnimationSummary>;
  moveAnimation(id: string, projectId: string): Promise<AnimationSummary>;
  duplicateAnimation(id: string, title: string, projectId?: string): Promise<AnimationSummary>;
  deleteAnimation(id: string): Promise<void>;
  usage(): Promise<{ usedBytes: number; limitBytes: number | null }>;
}
export const LIBRARY_STORE = new InjectionToken<LibraryStore>('LibraryStore');
```

- `HttpLibraryStore` implements it over `LibraryApi` of `projects/shared`; T2 adds the desktop one.
  Failures reject with `{ code, params }`.
- `CurrentAnimation` (`current-animation.ts`) knows whether the editor holds a saved animation —
  its id, version and project — or unsaved work.
- **Pages**: the library lists projects with their counts, and animations across projects with a
  search field; create, rename, duplicate and delete projects; for animations: open
  (`/editor/:animationId`), rename, move, duplicate, delete, each with a confirmation when it
  destroys. Lists load 50 at a time with a "Load more" button. Duplicate names are built by the
  client from translated keys (`library.copy_of`).
- **Saving** (Ctrl/⌘ `S`, and the Save button of the editor's header):
  - a visitor is asked to sign in or sign up; once signed in, saving goes on;
  - unsaved work asks for a project — pick one or create one — then `createAnimation`;
  - a saved animation calls `saveDocument` with its version;
  - `document.version_conflict` offers: reload the saved version, overwrite it — read the current
    version, then save —, or save a copy;
  - `quota.storage_exceeded` shows the usage, the limit and a link to the library;
  - success calls `engine.markSaved()`; failure keeps the work and says why.
- Opening another animation while there is unsaved work asks first.
- **Keys**: `library.`.

**Tests**: `HttpLibraryStore` against a mocked `ApiClient`; the visitor, first-save, save,
conflict — each of its three choices — and quota paths; opening with unsaved work; paging; axe on
the library pages and dialogs.

## H16 — Legal pages

- Texts: `i18n/legal/<code>/terms.md`, `privacy.md` and `notice.md`, in English and French. They
  use the placeholders `{{publisher}}`, `{{address}}`, `{{contact}}`, `{{director}}` and
  `{{host}}`, which the server fills from `LP_LEGAL_*`: a self-hosted server shows its own
  identity.
- Content: the **privacy policy** from [security-model.md](../security-model.md) — data collected,
  purposes, legal bases, retention, processors and their locations (D34, D35), rights and how to
  use them, the one cookie — the session's, strictly necessary, so no consent banner —, no
  third-party tracker. The **terms**: the service, the account, acceptable use, the creations
  belonging to their authors, the free plan and its quota, a beta's availability, liability,
  changes. The **legal notice**: publisher, publication director, host. Every statement the
  maintainer must write or confirm carries a `<!-- maintainer: … -->` comment, and the task lists
  them in its report.
- Server: `GET /i18n/legal/{code}/{page}.md`, `page` one of `terms`, `privacy`, `notice`;
  `text/markdown; charset=utf-8`, with an `ETag`; unknown code or page, `request.not_found`.
- App: `legal/legal-page.ts` on `legal/:page` loads the active language's text, else English,
  renders it with `marked`, sanitized by Angular. The shell's footer links the three pages on the
  hosted app; H7's sign-up page, which comes later, links the terms and the privacy policy.
- The catalogue check also requires the three files in every language, with the same placeholders.
- **Keys**: `legal.`.

**Tests**: placeholders filled from the configuration; `ETag` and `404`; the page rendered with its
headings and links; the footer links; the check failing on a missing file.
