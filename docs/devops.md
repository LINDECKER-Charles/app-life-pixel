# DevOps

The pipeline follows the one already used by the other projects of the shared VPS: build once,
promote the same artefact from staging to production, deploy over SSH behind the shared Caddy
edge. Nothing below exists yet; this is the target of the first CI pull request.

## Environments

| Environment | Where | Triggered by | Compose files | Configuration |
|---|---|---|---|---|
| local | developer machine | by hand | `compose.yaml` + `compose.override.yaml` | `.env` |
| CI | GitHub Actions | pull request, push to `dev` or `main` | — | `ENV_TEST` secret, if needed |
| staging | shared VPS | push to `dev`, once CI is green | `compose.yaml` + `compose.deploy.yaml` | `ENV_STAGING` secret |
| production | shared VPS | fast-forward of `main` | `compose.yaml` + `compose.deploy.yaml` | `ENV_PROD` secret |

| | local | staging | production |
|---|---|---|---|
| `COMPOSE_PROJECT_NAME` | `life-pixel` | `life-pixel-staging` | `life-pixel-prod` |
| Path on the host | — | `/opt/life-pixel-staging` | `/opt/life-pixel-prod` |
| Image tag | local build | `:<sha>`, `:staging` | `:<sha>`, `:prod` |
| MCP server name | `life-pixel-dev` | `life-pixel-staging` | `life-pixel` |
| Domains (D28) | `localhost` | `staging.lifepixel.tech`, `admin.staging.lifepixel.tech` | `lifepixel.tech`, `admin.lifepixel.tech` |
| Object storage (D35) | local, in Docker | a bucket in `fr-par` | a versioned bucket in `fr-par` |

`COMPOSE_PROJECT_NAME` isolates containers, volumes and networks, which is what lets staging and
production share the host. Images live in `ghcr.io/lindecker-charles/life-pixel/<image>`.

## Branches and promotion

```text
type/topic ──PR──► dev ──CI green──► test (fast-forward) ──► build images ──► deploy staging
                                       │
                         maintainer: fast-forward main to a validated test commit
                                       ▼
                                      main ──► promote images (no rebuild) ──► deploy production
                                       └──► tag vX.Y.Z ──► release: desktop, Android, npm, Docker
```

| Branch | Guarantees | Moved by |
|---|---|---|
| `type/topic` | nothing | its author |
| `dev` | integration; its tip may be red while CI runs | pull requests |
| `test` | the last `dev` commit whose CI is green — what staging runs | the pipeline, by fast-forward |
| `main` | what production runs | the maintainer, by fast-forward from `test` |

- **Every pull request targets `dev`**, contributors' and Dependabot's alike.
- **Every promotion is a fast-forward** (D23): the tree CI validated is bit for bit
  the one staging runs, then the one production runs. No merge commit creates a state nobody
  tested. The corollary: nothing is ever committed directly to `test` or `main`.
- **Production receives the images built from the exact commit `main` points to**, retagged by
  SHA — not whatever `:staging` happens to point to at that moment.
- **Protection**: `dev` requires a pull request and a green CI; `test` and `main` accept no direct
  push, only the fast-forwards described above.
- **`dev` is the default branch** (D14): new pull requests target it by default, and it is the
  only branch Dependabot security updates target, the one scheduled workflows run on, and the
  one where a workflow must exist to be dispatched by hand.

## Continuous integration

| Workflow | Trigger | Does |
|---|---|---|
| `ci.yml` | pull request, push to `dev` or `main` | orchestrates the stages below |
| `_verify.yml` | called | Rust: `fmt`, `clippy -D warnings`, tests, `cargo deny`, WebAssembly build and size budgets. Front-end: lint, format, type check, unit tests, build, catalogue parity. End-to-end tests of the critical path. Dockerfile and compose checks. `CLAUDE.md` identical to `AGENTS.md` |
| `_build.yml` | push to `dev`, after promotion to `test` | builds `app` and `admin`, pushes `:<sha>` and `:staging` to GHCR |
| `_promote.yml` | push to `main` | retags `:<sha>` as `:prod`, without rebuilding |
| `_deploy.yml` | after build or promotion | deploys to the VPS over SSH (below) |
| `security.yml` | pull request, push, weekly | CodeQL (`rust`, `javascript-typescript`, `actions`), dependency review, `cargo deny`, `npm audit` |
| `release.yml` | tag `vX.Y.Z` | Tauri builds — desktop installers (Windows, macOS, Linux) and the Android bundle for the Play Console —, `@life-pixel/player` to npm, Docker version tags, GitHub Release with provenance attestations |

The same checks run on the pull request and on the push that follows the merge: the second run
validates the real merge commit before it is promoted.

## Deploying on the shared VPS

The VPS hosts other projects. Its edge — `caddy-docker-proxy` on ports 80 and 443 with Let's
Encrypt —, the external networks `edge` and `observability`, and the observability stack belong
to the private `infra-vps` repository, which is deployed first. This repository only adds its
own stacks next to it.

The deploy job, per environment:

1. writes the `.env` file from the `ENV_STAGING` or `ENV_PROD` secret (mode 600);
2. syncs the repository on the host to the promoted commit;
3. checks that the `edge` network exists — and fails with "deploy infra-vps first" otherwise. It
   never writes to `/opt/edge` and never restarts Caddy, which serves every site of the host;
4. logs in to GHCR, then runs `docker compose pull` and `docker compose up -d --remove-orphans`
   with `COMPOSE_FILE=compose.yaml:compose.deploy.yaml`;
5. waits until every service reports healthy, and fails otherwise;
6. checks `https://<domain>/healthz` over TLS, as a warning only: a certificate that is still
   being issued is diagnosed, not fixed by restarting the edge.

What `compose.deploy.yaml` must declare:

- the public services (`server`, `admin`) join the external `edge` network with the
  `caddy: <domains>` and `caddy.reverse_proxy: "{{upstreams <port>}}"` labels; no `ports:`;
- a memory limit for every service, calibrated after a week of observation at about twice the
  observed peak — staging and production together must stay well below the host's memory; for
  Postgres, `shared_buffers` follows its limit;
- `server` and `admin` join the external `observability` network with the `prometheus.scrape`,
  `prometheus.port` and `prometheus.path` labels, and send traces to
  `http://otel-collector:4318`;
- never a `log` directive nor a `caddy.import: journal-acces` label: either would silently drop
  every other site of the host from the access log.

Services per environment: `server` (image `app`), `admin` (image `admin`), `postgres` with a
named volume. Object storage is an external S3-compatible bucket, one per environment (D16), on
Scaleway Object Storage in Paris (D35).

## Backups — required before production

`infra-vps` only backs up its own volumes. Life Pixel's production data needs (D16, D35):

- a nightly Postgres dump, encrypted before it leaves the host and sent to a bucket in Amsterdam
  (`nl-ams`), away from the live data, by credentials that cannot delete it — a compromised host
  cannot erase its own backups; copies are kept six months at most;
- versioning of the production bucket, whose replaced or deleted objects expire after 30 days;
- a restore rehearsed before launch, then periodically.

Staging data is disposable and not backed up.

## Secrets

GitHub secrets, following the shared deployment kit:

| Secret | Content |
|---|---|
| `STAGING_SSH_KEY`, `PROD_SSH_KEY` | private key dedicated to CI deployments |
| `STAGING_HOST`, `PROD_HOST` | the VPS |
| `STAGING_PATH`, `PROD_PATH` | `/opt/life-pixel-staging`, `/opt/life-pixel-prod` |
| `STAGING_SSH_USER`, `PROD_SSH_USER` | optional, `root` by default |
| `ENV_STAGING`, `ENV_PROD` | the full `.env` of the environment, as one multi-line secret |
| `ENV_TEST` | optional, for CI |

The `.env` of an environment carries the database URL, the object storage endpoint, bucket and
keys, the session secret, the SMTP settings (D34), the domains (`CADDY_DOMAINS`, `ADMIN_DOMAINS`),
the OpenTelemetry settings and, later, the billing keys. A versioned `.env.*.example` documents
each variable; the real files are never committed.

Release secrets, when the matching distribution lands (D32): the Tauri updater signing key, the
Apple Developer ID certificate and notarisation key, the Windows signing credentials, the Android
upload key and a Play Console service account. They live in a GitHub `release` environment that
requires the maintainer's approval. The updater key and the upload key also have an encrypted
offline copy: without the updater key, no installed desktop app could ever update again. npm
publishes through trusted publishing, without a token.

Enrolment starts about a month before M5 and M6: an organisation account needs a D-U-N-S number
first, and a new personal Play Console account must run a two-week closed test before it can
publish.

## Rollback

Every image keeps its immutable `:<sha>` tag. Rolling back means retagging `:prod` to a previous
SHA and deploying again. Database migrations stay backward compatible for one release — expand,
migrate, contract — so that rolling back an image never requires rolling back a schema.

## Dependabot

Weekly, grouped updates for `cargo` (`/`), `npm` (`/frontend`, `/player-js`), `github-actions`
(`/`) and `docker` (`/docker`), with pull requests against `dev`. Security updates follow the
default branch, which is `dev` (D14) — no `target-branch` needed. Dependabot alerts, security
updates, secret scanning with push protection and private vulnerability reporting are enabled
on the repository.
