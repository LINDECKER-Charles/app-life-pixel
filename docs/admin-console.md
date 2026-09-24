# Admin, support and metrics console

A separate application for the team, never for users: `frontend/projects/admin` for the
interface, `crates/admin-server` for its API. It has four jobs:

1. **Watch** staging and production: up, fast, failing, growing?
2. **Support** users: receive and handle their requests and complaints.
3. **Administer** accounts, plans, quotas and content.
4. **Understand** usage: what is used, by whom, what converts.

## Sources: build on the VPS's observability stack

The shared VPS already collects container metrics (cAdvisor), host metrics, the logs of every
container, the edge access log (per site, geolocated) and optionally traces. They are stored in
VictoriaMetrics, VictoriaLogs and VictoriaTraces, with alerting (vmalert, Alertmanager) and
Grafana on top. The console does not duplicate that stack: it reads it, and adds what only the
application knows (D17 in [decisions.md](decisions.md)).

| Data | Source | Read through |
|---|---|---|
| Technical metrics of our services | `/metrics` of `server` and `admin-server`, scraped by VictoriaMetrics | PromQL over HTTP |
| Container resources | cAdvisor, in VictoriaMetrics | PromQL |
| Logs | JSON on stdout → Vector → VictoriaLogs | LogsQL over HTTP |
| Traffic per domain | the edge access log, in VictoriaLogs | LogsQL |
| Accounts, plans, storage, requests | Postgres of each environment | the internal admin API of `server` |
| Product events | Postgres, hosted service only | the internal admin API of `server` |
| Firing alerts | vmalert and Alertmanager | their HTTP APIs |

Changes to agree with the `infra-vps` repository before the console can read everything:

- VictoriaLogs only listens on the observability stack's internal network today; the console needs
  read access to it.
- Read access to VictoriaMetrics should go through a proxy restricted to query endpoints (vmauth,
  for instance) rather than the full API, which includes deletion.
- Alert rules for Life Pixel are added in `infra-vps`, where every alert rule lives.

Deep investigations stay in Grafana: the console links to it with the right filters.

## Environments

One deployment per environment, like the application: the staging console administers staging
data, the production console production data, so an action can never land on the wrong database.
The monitoring views of both show both environments — the observability stack is shared — and a
permanent, colour-coded banner names the environment in use (staging in amber).

## Metrics catalogue

The approach is deliberately fine-grained. Technical metrics, exposed in Prometheus format with
low-cardinality labels:

| Metric | Labels |
|---|---|
| `http_requests_total`, `http_request_duration_seconds` | `route` (the template), `method`, `status_class` |
| `exports_total`, `export_duration_seconds`, `export_size_bytes` | `format`, `outcome` |
| `mcp_tool_calls_total`, `mcp_tool_duration_seconds` | `tool`, `outcome` |
| `storage_used_bytes` | `plan` |
| `quota_rejections_total` | `kind` |
| `auth_events_total` | `event` (`sign_in`, `sign_in_failed`, `token_created`…) |
| `db_pool_connections`, `db_query_duration_seconds` | `pool`, `query` (a name, never the SQL) |

Product metrics, computed from pseudonymous events: sign-ups, activation (first export), daily,
weekly and monthly active users, retention cohorts, conversion from free to paid, churn,
recurring revenue (from the billing provider), storage distribution against quotas, use of tools,
formats and MCP tools, split by platform, language and app version.

Events never contain pixel data or free text; they are first-party, kept 13 months, and described
in the privacy policy.

## Support requests

- Sent from the app (Help → Contact) with a category — bug, account, billing, data and privacy,
  abuse, other —, a message and an optional screenshot. The app version, platform, language and
  current screen are attached automatically; a creation only if the user attaches it.
- Statuses: new → in progress → waiting for the user → resolved → closed. Assignment, internal
  notes, replies sent by email and visible in the app.
- Data-protection requests (access, erasure) are a category of their own, with the legal deadline
  shown.
- Measured: open requests, their age, time to first answer, time to resolution.

## Administration

- **Users**: search; profile, plan, storage, activity, tokens; suspend and reactivate; export
  their data (right of access); delete them (right to erasure, object storage included).
- **Plans and quotas**: a per-user override always carries a reason and an expiry date.
- **Feature flags** and a maintenance banner.
- **Moderation**: a report queue, if public sharing is ever introduced.
- **Audit log**: every admin action — who, what, when, before and after — immutable and
  searchable.

## Access

- A dedicated host — `admin.lifepixel.tech`, `admin.staging.lifepixel.tech` (D28) —, not
  indexed, admin accounts only, a second factor mandatory, short sessions. An IP allow-list at
  the edge is possible.
- Least privilege: the console acts through the internal admin API of `server` over the private
  network. It holds no write access to the application's database.

## Interface principles

The console is only useful if it is pleasant to read at a glance.

- **Overview first.** One page answers "is everything fine?": the health of each service, error
  rate, latency, exports, MCP calls, storage, open requests, firing alerts. Every tile opens its
  detail.
- **Drill down in a straight line**: overview → area → detail → the log lines and the trace of a
  single request.
- **One time range and one environment selector**, shared by every view and kept in the URL: any
  view can be shared as a link.
- **Every chart answers a named question**, compares with the previous period and keeps one scale
  per axis.
- **An empty panel says why**: nothing in the range, or collection broken. A blank that looks like
  a dead stack is a bug.
- Keyboard navigation, dark mode, sortable tables exported as CSV.
- A dangerous action asks for confirmation, naming the environment and the target.
