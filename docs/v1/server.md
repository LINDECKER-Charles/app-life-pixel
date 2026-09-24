# Server — H3, H4, H13

`crates/server` is the binary `life-pixel-server`: the HTTP API under `/api/v1`, the MCP endpoint,
the i18n endpoint and the built app on one public listener; `/metrics` and the internal admin API
on two private ones. It translates HTTP into `service` calls and nothing more.

## H3 — Server foundation

### Layout

| Path | Content |
|---|---|
| `src/lib.rs` | the application, for the binary and for the crate's own integration tests; with `stack-tests`, the `testing` module |
| `src/main.rs` | the thin binary: subcommands `serve` (default), `openapi`, `migrate`, `healthcheck` |
| `src/config.rs` | `Config::from_env()`, every variable below, checked at start |
| `src/app.rs`, `src/state.rs` | the three routers and the shared state |
| `src/telemetry.rs` | logs, traces, metrics |
| `src/http/` | `problem.rs`, `request_id.rs`, `security_headers.rs`, `client_version.rs`, `rate_limit.rs`, `static_app.rs`, `i18n.rs` |
| `src/routes/` | `health.rs`; each feature adds its module |
| `src/openapi.rs` | the merged description |

Dependencies: `service`, `axum`, `tokio`, `tower`, `tower-http`, `utoipa`, `utoipa-axum`,
`metrics`, `metrics-exporter-prometheus`, `tracing`, `tracing-subscriber`,
`tracing-opentelemetry`, `opentelemetry-otlp`, `governor`, `semver`, `sha2`, `serde`, `serde_json`,
`clap`, `dotenvy`, `anyhow` in `main.rs` only.

- `serve` loads `.env` when present (development), reads the configuration, runs the migrations,
  then listens. For `ng serve`, H3 adds `frontend/projects/app/proxy.conf.json`, which forwards
  `/api` and `/mcp` to `http://localhost:8460` and keeps the browser's `Origin`. `openapi` prints
  the public description. `migrate` runs the migrations and exits. `healthcheck` requests `/healthz`
  on the local listener and exits 0 or 1: the images have no shell and no `curl`.

### Configuration

| Variable | Local value | Meaning |
|---|---|---|
| `LP_ENVIRONMENT` | `local` | `local`, `staging` or `production`, in logs and traces |
| `LP_HTTP_ADDR`, `LP_METRICS_ADDR`, `LP_ADMIN_API_ADDR` | `127.0.0.1:8460`, `:8461`, `:8462` | listeners; containers use `0.0.0.0:8080`, `:9090`, `:9091` |
| `LP_PUBLIC_URL` | `http://localhost:8460` | the origin, for links in emails and the CSRF origin check |
| `LP_ALLOWED_ORIGINS` | `http://localhost:4260` | more origins the CSRF check accepts — development only, for `ng serve`; empty on the hosts |
| `LP_APP_DIR`, `LP_I18N_DIR` | `frontend/dist/app/browser`, `i18n` | the built app; the catalogues |
| `LP_DATABASE_URL`, `LP_DATABASE_MAX_CONNECTIONS` | `postgres://life_pixel:local@127.0.0.1:5460/life_pixel`, `10` | |
| `LP_STORAGE_URL` | `s3://life-pixel-local` | `s3://<bucket>`, or `file:///<folder>` for self-hosting |
| `LP_S3_ENDPOINT`, `LP_S3_REGION`, `LP_S3_ACCESS_KEY_ID`, `LP_S3_SECRET_ACCESS_KEY`, `LP_S3_PATH_STYLE` | `http://127.0.0.1:5461`, `us-east-1`, `local`, `local`, `true` | S3 settings; Scaleway in `fr-par` on the hosts |
| `LP_SMTP_URL`, `LP_MAIL_FROM` | `smtp://127.0.0.1:5462`, `Life Pixel <no-reply@localhost>` | SMTP in URL form (D34) |
| `LP_SESSION_SECRET`, `LP_EVENTS_SECRET`, `LP_EXPORT_LINK_SECRET`, `LP_ADMIN_API_SECRET` | 64 hexadecimal characters each | HMAC keys; the last is shared with the admin server |
| `LP_PLAN_FREE_STORAGE_BYTES`, `LP_PLAN_FREE_MCP_CALLS_PER_DAY` | `100000000`, `1000` | the free plan (D31) |
| `LP_MIN_CLIENT_VERSIONS` | `web=0.0.0,desktop=0.0.0` | per platform |
| `LP_TRUSTED_PROXIES` | empty | networks whose `X-Forwarded-For` is believed |
| `LP_MCP_SERVER_NAME` | `life-pixel-dev` | the name MCP clients see (devops.md) |
| `LP_LEGAL_PUBLISHER`, `LP_LEGAL_ADDRESS`, `LP_LEGAL_CONTACT`, `LP_LEGAL_DIRECTOR`, `LP_LEGAL_HOST` | placeholders | [legal pages](accounts.md#h16--legal-pages) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | traces are sent only when set |
| `RUST_LOG` | `info` | |

A missing or invalid variable stops the start with a message naming it; secrets are never logged.

### Routers and middleware

- **Public** (`LP_HTTP_ADDR`): `GET /healthz` — `200 {"status":"ok"}` when the database answers
  within a second, `503` otherwise —; `/i18n/…`; `/api/v1/…`; `/mcp`; the built app for every other
  `GET` and `HEAD`.
- **Metrics** (`LP_METRICS_ADDR`): `GET /metrics`, Prometheus text.
- **Internal admin** (`LP_ADMIN_API_ADDR`): `/internal/admin/v1/…`
  ([support-admin.md](support-admin.md)).

On the public router, from the outside in:

1. **Request id**: `X-Request-Id` kept from a trusted proxy, generated otherwise (UUIDv7),
   returned, and attached to the request's span.
2. **Trace and metrics**: a span per request named after its route template (`MatchedPath`);
   `http_requests_total{route, method, status_class}` and
   `http_request_duration_seconds{route, method}`. An unmatched path counts as `route="unmatched"`:
   never a raw path.
3. **Security headers**, on every response:

| Header | Value |
|---|---|
| `Content-Security-Policy` | `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' blob:; worker-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'` |
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains`, when `LP_PUBLIC_URL` is `https` |
| `X-Content-Type-Options` | `nosniff` |
| `Referrer-Policy` | `no-referrer` |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=(), payment=()` |
| `Cross-Origin-Opener-Policy` | `same-origin` |

  Styles allow `'unsafe-inline'` for Angular's and Ionic's component styles; scripts stay strict.
  `connect-src` allows `blob:` for the timeline's preview, which plays a fresh export.
4. Under `/api/v1` and `/mcp`: **body limits** — 1 MiB, and the larger ones a route declares —
  and **rate limits** per route; under `/api/v1`, the **client version** check and problems for
  unknown routes and methods.

### Problems

```rust
pub struct Problem {
    pub status: StatusCode,
    pub code: &'static str,
    pub params: serde_json::Map<String, serde_json::Value>,
    pub retry_after: Option<u32>,
}
```

Every error answers `application/problem+json`, without a sentence:

```json
{ "type": "urn:life-pixel:problem:quota.storage_exceeded", "status": 409,
  "code": "quota.storage_exceeded", "params": { "used": 99000000, "limit": 100000000, "requested": 2000000 } }
```

A `Coded` error becomes a `Problem` through one table, code to status, the statuses of
[service.md](service.md#codes) and the server's own:

| Code | Params | Status |
|---|---|---|
| `request.malformed` | — | 400 |
| `request.not_found` | — | 404 |
| `request.method_not_allowed` | — | 405 |
| `request.too_large` | `maxBytes` | 413 |
| `request.unsupported_media_type` | — | 415 |
| `client.update_required` | `minimum` | 426 |
| `document.version_required` | — | 428 |
| `rate_limit.exceeded` | `retryAfterSeconds` | 429, with `Retry-After` |
| `internal.error` | — | 500, logged with the request id, nothing else returned |
| `service.unavailable` | — | 503 |

### Client version

Clients send `Life-Pixel-Client: <platform>/<version>` — `web/1.0.0`, `desktop/1.0.0`. Below the
minimum of `LP_MIN_CLIENT_VERSIONS`, the server answers `426` with `client.update_required`; a
request without the header, from a script, passes.

### Rate limits

In-process keyed limiters (`governor`): one server instance per environment. The client address
comes from `X-Forwarded-For` only when the peer is in `LP_TRUSTED_PROXIES`, taking the right-most
address not in it; otherwise it is the peer's.

| Policy | Key | Limit | Used by |
|---|---|---|---|
| `sign_in` | address; email | 10 a minute; 5 a minute | H5 |
| `sign_up` | address | 5 an hour | H5 |
| `password_reset` | address; email | 5 an hour each | H5 |
| `verification_resend` | account | 3 an hour | H5 |
| `events` | address | 60 a minute | H13 |
| `support_create` | account | 10 a day | H9 |
| `mcp` | token | 120 a minute | A3 |
| `api` | account, else address | 600 a minute | every other API route |

### Static app and i18n

- The built app is served from `LP_APP_DIR`; a `GET` that finds no file and does not start with
  `/api/`, `/i18n/` or `/mcp` gets `index.html`. Files named with a content hash (`-XXXXXXXX.js`,
  `.css`) are `public, max-age=31536000, immutable`; everything else — `index.html`, `/engine/`,
  `life-pixel.js` — is `no-cache` with an `ETag`.
- `/i18n/languages.json` and `/i18n/{code}.json` come from `LP_I18N_DIR`, read at start into
  memory, with a strong `ETag` (SHA-256), `304` on `If-None-Match`, and `Cache-Control: no-cache`.
  A code absent from `languages.json` is `request.not_found`; nothing outside the folder is ever
  read.

### Telemetry

- Logs: JSON on stdout, one event per line — `timestamp`, `level`, `target`, `message`,
  `request_id`, `route`, `method`, `status`, `duration_ms` —, filtered by `RUST_LOG`; never an
  email address, a token, a password or a document.
- Traces: OTLP over HTTP to `OTEL_EXPORTER_OTLP_ENDPOINT` (`http://otel-collector:4318` on the
  hosts), service `life-pixel-server`, `deployment.environment` from `LP_ENVIRONMENT`.
- Metrics: the catalogue of [admin-console.md](../admin-console.md); each feature registers its
  own, with low-cardinality labels.

### OpenAPI and the typed client

- Each route module builds a `utoipa_axum::router::OpenApiRouter` with `routes!(…)`; `openapi.rs`
  merges them. Request and response types derive `ToSchema`, in camelCase; the security schemes are
  the session cookie with its CSRF header, and bearer tokens.
- `life-pixel-server openapi` prints the description with sorted keys; it is committed as
  `crates/server/openapi.json`.
- `frontend/tools/generate-api.mjs` — `npm run api:generate` — writes that file, then runs
  `openapi-typescript` into `projects/shared/src/lib/api/schema.d.ts`; H11 adds the admin server's
  description. CI's `api` job regenerates both and fails on any difference.
- `projects/shared/src/lib/api/api-client.ts`: `ApiClient`, the only user of `HttpClient`, typed by
  path and method from `schema.d.ts`; it sends `Life-Pixel-Client`, lets H5 add the CSRF header,
  and turns a problem into an `ApiProblem { status, code, params }` it throws. Each feature adds a
  service beside it — `auth-api.ts`, `library-api.ts` —, and an ESLint `no-restricted-imports`
  rule keeps `@angular/common/http` out of every other folder.

**Tests**: configuration errors named; the problem shape and the status of every code, from one
table-driven test; the security headers on app, API and error responses; `426`; `429` with
`Retry-After`; the i18n `ETag` and `304`; caching headers and the `index.html` fallback, never for
`/api/`; the request id; `/metrics` with route templates only; the `openapi` output stable.

## H4 — Postgres and object storage

### Schema

`crates/server/migrations/`, run by `sqlx::migrate!`; one file per change, named
`<UTC timestamp>_<topic>.sql`; each migration stays compatible with the previous release's code
(expand, migrate, contract — devops.md). Queries are written with `sqlx::query_as` and checked by
the tests, not at compile time: no database is needed to build.

```sql
create extension if not exists citext;
create extension if not exists pg_trgm;

create table accounts (          -- H5 adds the identity columns
  id uuid primary key,
  plan text not null default 'free',
  storage_used_bytes bigint not null default 0 check (storage_used_bytes >= 0),
  created_at timestamptz not null default now()
);

create table projects (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  name text not null,
  created_at timestamptz not null,
  updated_at timestamptz not null
);
create index projects_account_updated on projects (account_id, updated_at desc, id desc);

create table animations (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  project_id uuid not null references projects (id) on delete cascade,
  title text not null,
  width integer not null,
  height integer not null,
  frame_count integer not null,
  document_key text not null unique,
  document_bytes bigint not null,
  version bigint not null,
  created_at timestamptz not null,
  updated_at timestamptz not null
);
create index animations_account_updated on animations (account_id, updated_at desc, id desc);
create index animations_project_updated on animations (project_id, updated_at desc, id desc);
create index animations_title_trgm on animations using gin (title gin_trgm_ops);
```

### The hosted library store

`storage::HostedLibraryStore` implements `LibraryStore` for `Owner::Account`.

- Lists use keyset pagination on `(updated_at, id)` descending; search is `title ilike` with `%`
  and `_` escaped, served by the trigram index.
- A document is stored at `documents/<account>/<animation>/<uuid>.json` — a new key per write:
  1. the object is written first, outside any transaction;
  2. one transaction locks the account's row and the animation's (`for update`), checks the version
     and the quota rule, points the animation at the new key with `version + 1`, and moves
     `storage_used_bytes` by the difference;
  3. the old object is deleted. If step 2 fails, the new object is deleted; if a deletion fails, it
     is logged and the sweeper collects it later. Usage stays exact in every case.
- **Object storage** is built from `LP_STORAGE_URL` through `object_store`: `AmazonS3Builder` with
  the `LP_S3_*` settings, or `LocalFileSystem` for a `file://` URL, the self-hosting default.
- **Sweeper**: every 6 hours, objects under `documents/` older than 24 hours that no row references
  — checked by batches of 1,000 keys — are deleted; `storage_orphans_deleted_total` counts them. H9
  adds its `support/` prefix.
- Metrics: `db_pool_connections{pool, state}`; `db_query_duration_seconds{query}` with a query
  name, never its SQL; `storage_used_bytes{plan}`, from the accounts' usage.

### Test databases

Tests that need the local stack are integration tests with `required-features = ["stack-tests"]`.
`life_pixel_server::testing` (feature `stack-tests`) gives them:

- `TestDatabase::create()`: on the server of `LP_TEST_DATABASE_URL` — the local stack, in CI too,
  so that `init.sh` has created the roles —, creates `lp_test_<16 random hexadecimal digits>`
  owned by `life_pixel`, records its creation time as the database comment,
  migrates it, and returns a pool; `drop()` removes it `with (force)`. Creating one also removes
  the `lp_test_` databases whose comment is more than an hour old: the leftovers of crashed runs,
  never those of a run in progress.
- `TestStorage::create()`: the S3Mock bucket under a random prefix.
- `TestMailbox::new()`: a random recipient at `test.life-pixel.invalid`, and the Mailpit API
  searched for it.

- `life-pixel-server test-database create` prints the URL of a new test database, and
  `test-database drop <url>` removes it: the subcommand, compiled with `stack-tests` only, serves
  H17's end-to-end setup.

`#[sqlx::test]` is not used: as of sqlx 0.8 it names a test's database after the test's path, so
two worktrees running the same test on one server drop each other's database.

**Tests** (`stack-tests`): H1's contract suite against `HostedLibraryStore`; two concurrent writes
on one version, one wins; a failure injected after the object write leaves usage exact, and the
sweeper deletes the orphan; search through the trigram index; the migrations on an empty database.

## H13 — Product events

```sql
create table product_events (
  id bigint generated always as identity primary key,
  name text not null,
  subject text,                    -- keyed hash of the account, or null
  properties jsonb not null default '{}',
  platform text,
  app_version text,
  language text,
  occurred_at timestamptz not null
);
create index product_events_name_time on product_events (name, occurred_at);
create index product_events_subject_time on product_events (subject, occurred_at);
```

- **Subject**: the first 32 hexadecimal digits of HMAC-SHA256(`LP_EVENTS_SECRET`, account id):
  stable for an account, meaningless without the secret. No email, no pixel, no free text.
- **Sink**: `PostgresEventSink` puts events on a bounded channel of 10,000; a task inserts them by
  batches, every second or every 100; when the channel is full, the event is dropped and
  `events_dropped_total` counts it. A failure never reaches the use case.
- **`POST /api/v1/events`** (rate limit `events`), for the app's allow-list only:

```json
{ "name": "export_completed", "properties": { "format": "gif", "bytes": 8421 },
  "platform": "web", "appVersion": "1.0.0", "language": "fr" }
```

`202`, the subject taken from the session if any. `format` takes a value of
  [service.md](service.md#product-events) and `bytes` a non-negative integer, which the server turns
  into the event's size class, with `source` set to `app`: no size class is written in TypeScript.
  Any other name, property or value is `request.malformed`. H13 provides the app's hosted
  `EXPORT_OBSERVER` (U5), which posts this event; the desktop keeps U5's default, which sends
  nothing.
- **Purge**: every day, events older than 395 days — 13 months — are deleted.
- **Aggregates** (`events::queries`), for the admin API: sign-ups per day; activation — accounts
  whose first `export_completed` falls in the period; distinct active subjects per day, week and
  month; exports by format and source; MCP calls by tool and outcome; storage distribution by size
  class, from `accounts.storage_used_bytes`.

**Tests**: the allow-list; the same subject for one account, another for another secret; batching
and the drop counter; the purge; each aggregate on a fixed set of events.
