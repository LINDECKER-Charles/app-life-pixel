# Support and administration — H9, H10, H11, H12, H17

Support requests from the app, and the separate admin console of
[admin-console.md](../admin-console.md) in its V1 scope: monitoring of both environments, users,
support requests, audit log.

```text
admin console ──► admin server ──► internal admin API of server ──► Postgres, object storage
                       └──────────► VictoriaMetrics (query-only proxy), VictoriaLogs, Alertmanager
```

## H9 — Support requests

### Data

```sql
create table support_requests (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  category text not null
    check (category in ('bug', 'account', 'billing', 'data_protection', 'abuse', 'other')),
  status text not null default 'new'
    check (status in ('new', 'in_progress', 'waiting_for_user', 'resolved', 'closed')),
  context jsonb not null,            -- appVersion, platform, language, screen
  screenshot_key text,
  assigned_to text,                  -- an admin id, set through the admin API
  created_at timestamptz not null,
  updated_at timestamptz not null,
  first_response_at timestamptz,
  resolved_at timestamptz
);
create index support_requests_status on support_requests (status, updated_at desc);
create index support_requests_account on support_requests (account_id, updated_at desc);

create table support_messages (
  id uuid primary key,
  request_id uuid not null references support_requests (id) on delete cascade,
  author text not null check (author in ('user', 'team')),
  admin_id text,
  body text not null,
  internal boolean not null default false,
  created_at timestamptz not null
);
```

### Routes

| Route | Body | Answer |
|---|---|---|
| `POST /api/v1/support-requests` | `multipart/form-data`: `category`, `message`, `context` (JSON), optional `screenshot` | `201` SupportRequest |
| `GET /api/v1/support-requests` | `?cursor&limit` | `200` Page of summaries |
| `GET /api/v1/support-requests/{id}` | — | `200` SupportRequest |
| `POST /api/v1/support-requests/{id}/messages` | `{ body }` | `201` the message |

- SupportRequest: `{ id, category, status, hasScreenshot, createdAt, updatedAt, messages: [{ id,
  author: "user" | "team", body, createdAt }] }` — internal notes never appear, and neither does
  the screenshot: a user's upload is never served back.
- The request's message is its first message. A reply by the user moves `waiting_for_user` back to
  `in_progress`; a closed request refuses replies (`support.request_closed`).
- `context`: `appVersion`, `platform`, `language`, and `screen` — the route's template
  (`/editor/:animationId`), never a URL with ids.
- **Screenshot**: PNG or JPEG, at most `SCREENSHOT_MAX_BYTES` and `SCREENSHOT_MAX_SIDE`, decoded
  with the `image` crate under those limits, re-encoded as PNG — metadata dropped —, stored at
  `support/<request>/screenshot.png`. The route's body limit is 6 MiB. The sweeper's `support/`
  prefix collects screenshots whose request is gone.
- Rate limit `support_create`; records `support_request_created` with the category.
- `service::support` holds the ports `SupportStore` and `ScreenshotStore` and the use cases;
  `server` implements them.

Codes: `support.request_not_found` (404), `support.category` (422), `support.message_length`
(`min`, `max`; 422), `support.screenshot` (`maxSide`, `maxBytes`; 422), `support.request_closed`
(409).

### App

`frontend/projects/app/src/app/support/`, which knows the person through H7's `SessionStore`:
`support-page.ts` lists the person's requests and holds
"New request" — category, message with a character counter, optional screenshot checked for type
and size before upload; the context is attached without asking —; `support-request-page.ts` shows
the thread and a reply field. Signed out, the page explains that support needs an account; the
desktop, which has no account, does not route it (T2). **Keys**: `support.`, and
`email.support_reply.*`, sent by H10.

**Tests**: each route and code (`stack-tests`); a screenshot re-encoded without its metadata, and
one whose header claims 100,000 pixels refused before decoding; internal notes absent from the
user's view; the pages with a mocked client; axe.

## H10 — Internal admin API

On `LP_ADMIN_API_ADDR`, under `/internal/admin/v1`, never routed by the edge. Every request carries
`Authorization: Bearer <LP_ADMIN_API_SECRET>`, compared in constant time, and the acting admin's
`X-Admin-Id` and `X-Admin-Email`; without them, `401 admin.unauthenticated`.

### Audit log

```sql
create table audit_log (
  id bigint generated always as identity primary key,
  admin_id text not null,
  admin_email text not null,
  action text not null,              -- user.suspend, support.reply, …
  target_type text not null,
  target_id text not null,
  reason text,
  before jsonb,
  after jsonb,
  at timestamptz not null
);
create function audit_log_append_only() returns trigger language plpgsql as $$
begin raise exception 'audit_log is append-only'; end $$;
create trigger audit_log_append_only before update or delete on audit_log
  for each row execute function audit_log_append_only();
```

Every change below writes its entry in the same transaction; reading a user's data export writes
one too. After an erasure, the entry keeps the account id and the reason, never the address.

### Routes

| Route | Answer |
|---|---|
| `GET /users?q&status&cursor&limit` | users found by address or id: `id`, `email`, `emailVerified`, `plan`, `status`, `storageUsedBytes`, `createdAt`, `lastSeenAt` |
| `GET /users/{id}` | the same, plus language, project and animation counts, events of the last 30 days by name, sessions, support requests; A3 adds tokens |
| `POST /users/{id}/suspend`, `POST /users/{id}/reactivate` | `{ reason }` → `204`; suspending ends the sessions |
| `GET /users/{id}/export` | the zip of H6 |
| `DELETE /users/{id}` | `{ reason }` → `204`, H6's deletion |
| `GET /support-requests?status&category&assignee&cursor&limit` | the queue, with the account's address and each request's age |
| `GET /support-requests/{id}` | the thread with internal notes, the context, `hasScreenshot` |
| `GET /support-requests/{id}/screenshot` | the PNG |
| `PATCH /support-requests/{id}` | `{ status?, assignedTo? }` → the request |
| `POST /support-requests/{id}/messages` | `{ body, internal }` → the message; a reply is emailed (`SupportReply`, linking `/support/{id}`), sets `first_response_at`, and moves the request to `waiting_for_user` |
| `GET /metrics/product?from&to` | H13's aggregates, plus open requests by status and the medians of their age, of the time to a first answer and of the time to resolution |
| `GET /audit-log?adminId&action&targetId&cursor&limit` | entries, newest first |
| `GET /environment` | `{ environment, version }` |

`life-pixel-server admin-openapi` writes this API's description to
`crates/server/admin-openapi.json`, committed and checked by the `api` job: H11 builds on it.

**Tests** (`stack-tests`): the secret and the identity headers required; each route; each change
audited in its transaction, a failure leaving neither the change nor the entry; the log refusing
`update` and `delete`; the reply email; the metrics on fixed data.

## H11 — Admin server

`crates/admin-server`, binary `life-pixel-admin-server`, depends on no domain crate.

| Path | Content |
|---|---|
| `src/lib.rs`, `src/main.rs` | the application; the binary's subcommands `serve`, `openapi`, `migrate`, `healthcheck`, `create-admin`, `disable-admin`, and `test-database` with `stack-tests`, as H4's |
| `src/config.rs`, `src/app.rs`, `src/telemetry.rs` | as in the server |
| `src/http/` | problems, security headers, CSRF, sessions, static console |
| `src/admins/` | admin accounts, TOTP, sign-in routes |
| `src/relay.rs` | the relay to the internal admin API |
| `src/monitoring/` | `metrics.rs`, `logs.rs`, `alerts.rs`, `links.rs` |
| `migrations/` | its own database |

### Configuration

| Variable | Local value | Meaning |
|---|---|---|
| `LPA_ENVIRONMENT` | `local` | the environment this console administers, named in its banner |
| `LPA_HTTP_ADDR`, `LPA_METRICS_ADDR` | `127.0.0.1:8463`, `:8464` | containers: `0.0.0.0:8080`, `:9090` |
| `LPA_PUBLIC_URL`, `LPA_APP_DIR`, `LPA_I18N_DIR` | `http://localhost:8463`, `frontend/dist/admin/browser`, `i18n` | |
| `LPA_ALLOWED_ORIGINS` | `http://localhost:4263` | more origins the CSRF check accepts, for `ng serve`; empty on the hosts |
| `LPA_DATABASE_URL` | `postgres://life_pixel_admin:local@127.0.0.1:5460/life_pixel_admin` | |
| `LPA_SESSION_SECRET`, `LPA_TOTP_KEY` | 64 hexadecimal characters each | CSRF key; key encrypting the TOTP secrets |
| `LPA_SERVER_ADMIN_API_URL`, `LPA_SERVER_ADMIN_API_SECRET` | `http://127.0.0.1:8462/internal/admin/v1`, the server's secret | |
| `LPA_ENVIRONMENTS` | `staging,production` | the environments its monitoring shows |
| `LPA_VICTORIAMETRICS_URL`, `LPA_VICTORIALOGS_URL`, `LPA_ALERTMANAGER_URL`, `LPA_GRAFANA_URL` | empty | empty: the page says the source is not configured |
| `LPA_METRICS_SELECTOR_<ENV>`, `LPA_CONTAINERS_SELECTOR_<ENV>`, `LPA_LOGS_SELECTOR_<ENV>`, `LPA_ALERTS_FILTER` | empty | label selectors that `infra-vps` gives |

### Admin accounts

```sql
create table admins (
  id uuid primary key,
  email citext not null unique,
  password_hash text not null,          -- Argon2id, as H5
  totp_secret bytea not null,           -- XChaCha20-Poly1305 with LPA_TOTP_KEY
  totp_last_step bigint not null default 0,
  created_at timestamptz not null,
  disabled_at timestamptz
);
create table admin_sessions (
  token_hash bytea primary key,
  admin_id uuid not null references admins (id) on delete cascade,
  created_at timestamptz not null,
  last_seen_at timestamptz not null,
  expires_at timestamptz not null
);
```

- `POST /api/admin/v1/auth/sign-in` `{ email, password, code }`: a wrong address, password or code
  all answer `admin.invalid_credentials`; 5 attempts a minute per address and per email. TOTP:
  SHA-1, 6 digits, 30 seconds, one step of drift either way, and a step at or below
  `totp_last_step` refused, so that a code works once.
- The session cookie `__Host-lpa_session` is `SameSite=Strict`; a session ends after 30 minutes
  idle or 8 hours in all. CSRF as H5, with `LPA_SESSION_SECRET` and
  `LPA_ALLOWED_ORIGINS` (`admin.csrf`).
  `GET /api/admin/v1/auth/session` → `{ admin: { id, email }, csrfToken, environment }`.
- `create-admin --email <address>` reads the password twice without echo — or once from
  `--password-stdin`, for tests —, generates a 20-byte TOTP secret, and prints the
  `otpauth://totp/…?secret=…&issuer=Life%20Pixel` URI to add to an authenticator.
  `disable-admin --email <address>` disables one.

### Relay and monitoring

- `/api/admin/v1/users/…`, `/support-requests/…`, `/metrics/product` and `/audit-log` are relayed to
  `LPA_SERVER_ADMIN_API_URL` with the secret and the admin's identity, bodies streamed both ways;
  the server's problems pass through unchanged.
- Monitoring only reads, and the browser never sends a query: it names a panel, the admin server
  builds the query, `{SEL}` standing for the environment's selector.

| Panel | PromQL |
|---|---|
| `up` | `min(up{SEL})` |
| `requests` | `sum(rate(http_requests_total{SEL}[5m]))` |
| `errors` | `sum(rate(http_requests_total{SEL,status_class="5xx"}[5m])) / sum(rate(http_requests_total{SEL}[5m]))` |
| `latency` | `histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket{SEL}[5m])))`, and 0.5, 0.99 |
| `slow_routes` | `topk(10, histogram_quantile(0.95, sum by (le, route) (rate(http_request_duration_seconds_bucket{SEL}[1h]))))` |
| `exports`, `mcp`, `auth` | `sum by (…) (increase(exports_total{SEL}[$range]))`, the same for `mcp_tool_calls_total` and `auth_events_total` |
| `storage`, `quota` | `sum(storage_used_bytes{SEL})`, `sum(increase(quota_rejections_total{SEL}[$range]))` |
| `memory`, `cpu` | `container_memory_working_set_bytes{SEL}`, `rate(container_cpu_usage_seconds_total{SEL}[5m])`, with the containers' selector |

- Routes: `GET /api/admin/v1/monitoring/overview?env&from&to` — the tiles; `GET
  /monitoring/series?env&panel&from&to&step` — a panel's series and the previous period's;
  `GET /logs?env&level&q&from&to&limit` — LogsQL `{LOGS_SEL} AND level:<level> AND "<q>"`, the text
  escaped, 500 lines at most; `GET /alerts?env` — Alertmanager's `/api/v2/alerts` with
  `LPA_ALERTS_FILTER`; `GET /links/grafana?env&requestId` — a Grafana Explore link to that request's
  logs and trace. A source not configured answers `monitoring.not_configured` (503), one that fails
  `monitoring.unavailable` (502).
- `life-pixel-admin-server openapi` merges its own routes with `crates/server/admin-openapi.json`,
  paths moved under `/api/admin/v1`, into `crates/admin-server/openapi.json`;
  `npm run api:generate` generates `projects/shared/src/lib/admin-api/schema.d.ts` from it.

**Tests**: TOTP — valid, drifted by a step, replayed, wrong —; idle and absolute session expiry;
CSRF; the relay's headers and streaming; every panel's query with its selector; LogsQL escaping;
monitoring against fake VictoriaMetrics, VictoriaLogs and Alertmanager servers; `create-admin`.

## H12 — Admin console

`frontend/projects/admin` (`ng generate application admin --prefix=lp`), served by the admin
server; `npm run start:admin` on port 4263 proxies the API to 8463; `npm run build` builds both
applications. Charts use uPlot.

| Route | Page |
|---|---|
| `sign-in` | address, password, code |
| `''` | overview: for each environment of `LPA_ENVIRONMENTS`, tiles for health, error rate, p95 latency, requests, exports, MCP calls, storage, open support requests, firing alerts; each opens its detail |
| `monitoring/:panel` | a panel's chart, with the previous period |
| `logs` | filters by level, text and time; each line's request id opens its logs and its Grafana trace |
| `alerts` | firing alerts |
| `users`, `users/:id` | search, sortable table, detail; suspend, reactivate, export, delete — each with a reason and a confirmation naming the environment and the target |
| `support`, `support/:id` | the queue, filtered and sorted; the thread with notes, reply or note, status, assignment, screenshot, context; a `data_protection` request shows its legal deadline, one month after its creation |
| `audit` | the log, searchable |

- A banner names the environment on every page, amber for staging. The environment and the time
  range live in the URL (`?env=production&from=now-24h&to=now`), shared by every view, so any view
  is a link.
- Every chart's title is the question it answers; an empty panel says why — nothing in the range,
  or a source unreachable or not configured.
- Tables sort and export to CSV. Keyboard, dark mode, reduced motion, contrast AA.
- **Keys**: `admin.`.

**Tests**: services against a mocked client; the URL state; the banner; CSV; confirmations naming
their target; empty states.

## H17 — Hosted end to end

`frontend/e2e/hosted/`, Playwright project `hosted`, `npm run e2e:hosted`, on the local stack.

- **Setup**: `npm run build` has run; the global setup creates the server's and the admin server's
  test databases with the `test-database` subcommands of H4 and H11, starts both binaries with a
  200,000-byte quota, and creates an admin with `create-admin --password-stdin`, reading the TOTP
  secret from the printed URI; the teardown drops the databases.
- **Helpers**: `mailpit.ts` finds a recipient's last message and its link; `totp.ts` computes codes
  with `otpauth`.
- **Journeys**: a visitor draws, saves, signs up, comes back to the editor and saves into a new
  project, then verifies the address from the email; library search, rename, duplicate, delete;
  two pages saving one animation, the second choosing "save a copy"; the quota reached and
  explained; the data export downloaded and holding `library.json` and the documents; a password
  reset by email; a support request with a screenshot, answered from the admin console, the reply
  seen in the app and received by email; a user suspended then reactivated; an account deleted,
  after which sign-in fails.
- `@axe-core/playwright` on every page these journeys open, no serious or critical violation.
- A passing H17 means M3 is done.
