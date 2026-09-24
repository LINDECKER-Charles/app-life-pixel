# Operations — H14, H15

Images, the deployment overlay for the shared VPS, the delivery workflows and backups, as
[devops.md](../devops.md) describes them. Agents write and test all of it locally; only the
maintainer pushes, configures the hosts and deploys.

## H14 — Images and deployment

### Images

Images live in `ghcr.io/lindecker-charles/life-pixel/<image>`. Every base image is pinned by
digest.

| File | Stages |
|---|---|
| `docker/app.Dockerfile` | `rust` — `rust:1.98-bookworm`: installs `wasm-bindgen-cli` at the version given by the `WASM_BINDGEN_VERSION` argument, runs `cargo xtask build-editor` and `cargo build --release --locked -p life-pixel-server`; `web` — `node:24.21-bookworm-slim`: copies `player-js/` — the app depends on it — then runs `npm ci`, copies the engine's outputs from `rust`, and with `LP_ENGINE_PREBUILT=1` runs `npm run build:app`; `runtime` — `gcr.io/distroless/cc-debian12:nonroot`: the binary in `/usr/local/bin`, the app in `/srv/app`, `i18n/` in `/srv/i18n` |
| `docker/admin.Dockerfile` | the same without the engine: `life-pixel-admin-server`, `npm run build:admin`, the console in `/srv/admin` |

- The runtime sets `LP_HTTP_ADDR=0.0.0.0:8080`, `LP_METRICS_ADDR=0.0.0.0:9090`,
  `LP_ADMIN_API_ADDR=0.0.0.0:9091`, `LP_APP_DIR`, `LP_I18N_DIR` (the admin image its `LPA_`
  equivalents), runs as `nonroot`, exposes 8080, and declares
  `HEALTHCHECK CMD ["/usr/local/bin/life-pixel-server", "healthcheck"]`.
- `.dockerignore` at the root keeps out `target/`, every `node_modules/`, `frontend/dist/`, `.git/`
  and the `.env` files.
- The images build the front-ends with F3's `build:app` and H12's `build:admin` scripts.

### Compose files

`compose.yaml` gains the two services every environment runs:

```yaml
  server:
    image: ghcr.io/lindecker-charles/life-pixel/app:${IMAGE_TAG:-local}
    environment:                 # one line per LP_ variable of server.md, from the project's .env
      LP_ENVIRONMENT: ${LP_ENVIRONMENT}
      LP_DATABASE_URL: ${LP_DATABASE_URL}
      # …
    depends_on:
      postgres: { condition: service_healthy }
    healthcheck:
      test: ["CMD", "/usr/local/bin/life-pixel-server", "healthcheck"]
      interval: 15s
      timeout: 5s
      retries: 5
      start_period: 30s
    deploy:
      resources:
        limits: { memory: 512M }
    restart: unless-stopped
  admin:
    image: ghcr.io/lindecker-charles/life-pixel/admin:${IMAGE_TAG:-local}
    environment:                 # one line per LPA_ variable of support-admin.md
      LPA_ENVIRONMENT: ${LPA_ENVIRONMENT}
      LPA_DATABASE_URL: ${LPA_DATABASE_URL}
      # …
    depends_on:
      server: { condition: service_healthy }
    healthcheck: { test: ["CMD", "/usr/local/bin/life-pixel-admin-server", "healthcheck"] }
    deploy:
      resources:
        limits: { memory: 256M }
    restart: unless-stopped
```

Each service lists its own variables under `environment`, interpolated from the project's `.env`,
and no service uses `env_file`: a container receives only its own variables, so the admin server
never sees `LP_DATABASE_URL` and holds no access to the application's database
([admin-console.md](../admin-console.md)).

`compose.override.yaml` gives them, locally, the `app` profile — so that `docker compose up`
starts only the stack unless `--profile app` is given —, their `build` sections and ports on
`127.0.0.1`, and replaces, under `environment`, what differs inside a container: the image's
listeners and folders, the stack's addresses (`postgres:5432`, `objectstore:9090`, `mail:1025`),
and `LPA_SERVER_ADMIN_API_URL=http://server:9091/internal/admin/v1`. One `.env` serves the host and
the containers.

`compose.deploy.yaml`, for staging and production, follows the shared-VPS rules of AGENTS.md:

```yaml
services:
  server:
    networks: [default, edge, observability]
    environment:
      OTEL_EXPORTER_OTLP_ENDPOINT: http://otel-collector:4318
    labels:
      caddy: ${CADDY_DOMAINS}
      caddy.reverse_proxy: "{{upstreams 8080}}"
      prometheus.scrape: "true"
      prometheus.port: "9090"
      prometheus.path: /metrics
  admin:
    networks: [default, edge, observability]
    environment:
      OTEL_EXPORTER_OTLP_ENDPOINT: http://otel-collector:4318
    labels:
      caddy: ${ADMIN_DOMAINS}
      caddy.reverse_proxy: "{{upstreams 8080}}"
      prometheus.scrape: "true"
      prometheus.port: "9090"
      prometheus.path: /metrics
  postgres:
    command: ["postgres", "-c", "shared_buffers=128MB"]
networks:
  edge: { external: true }
  observability: { external: true }
```

- No `ports:`; never a `log` directive nor a `caddy.import: journal-acces` label. The internal
  admin API, port 9091, is reached by the admin server over the project's `default` network and
  never routed by the edge. Docker cannot close a port to one network, so containers of other
  projects on `edge` or `observability` could reach it: it answers only to
  `LP_ADMIN_API_SECRET`, 256 random bits compared in constant time.
- Memory limits start at 512 MiB for `server` and `postgres`, 256 MiB for `admin`, and are
  recalibrated after a week at about twice the observed peak.
- `.env.staging.example` and `.env.prod.example` list every variable of both binaries, plus
  `COMPOSE_PROJECT_NAME`, `CADDY_DOMAINS`, `ADMIN_DOMAINS`, the Postgres passwords and, for
  production, `COMPOSE_PROFILES=backup` and H15's variables — with placeholders, never values.
  They never set the listeners (`LP_HTTP_ADDR`, `LP_METRICS_ADDR`, `LP_ADMIN_API_ADDR`) nor the
  folders (`LP_APP_DIR`, `LP_I18N_DIR`), nor the admin server's: the images' values apply.

### Delivery workflows

`ci.yml` gains, after `verify`, on pushes only:

| Job | When | Does |
|---|---|---|
| `promote-test` | push to `dev` | `git push origin <sha>:refs/heads/test` with the `PROMOTION_DEPLOY_KEY` secret, which the `test` ruleset lets through; fails unless it is a fast-forward |
| `build` | after `promote-test` | calls `_build.yml` |
| `deploy-staging` | after `build` | calls `_deploy.yml` for `staging` |
| `promote-images` | push to `main` | calls `_promote.yml` |
| `deploy-production` | after `promote-images` | calls `_deploy.yml` for `production` |

- `_build.yml`: reads the `wasm-bindgen` version from `Cargo.lock`, builds `app` and `admin` with
  Buildx and the GitHub Actions cache, pushes `:<sha>` and `:staging`, and
  attests their provenance (`actions/attest-build-provenance`, pushed to the registry).
  Permissions: `contents: read`, `packages: write`, `id-token: write`, `attestations: write`.
- `_promote.yml`: `docker buildx imagetools create` tags `:<sha>` as `:prod`, without rebuilding.
- `_deploy.yml`, in the GitHub environment of its target, runs `scripts/deploy/deploy.sh` on the
  host over SSH (`<ENV>_SSH_KEY`, `<ENV>_HOST`, `<ENV>_PATH`, `<ENV>_SSH_USER`), in six steps:
  1. write `.env` from `ENV_STAGING` or `ENV_PROD`, mode 600;
  2. clone the repository into the path if needed, then fetch and check out `<sha>`;
  3. fail with "deploy infra-vps first" unless the `edge` and `observability` networks exist —
     never touching `/opt/edge` nor restarting Caddy;
  4. log in to GHCR with the job's token, then, with `COMPOSE_FILE=compose.yaml:compose.deploy.yaml`
     and `IMAGE_TAG=<sha>`, `docker compose pull` and `docker compose up -d --remove-orphans`;
  5. wait up to 180 seconds for every service to be healthy, else print the unhealthy services'
     logs and fail;
  6. request `https://<domain>/healthz` and only warn on failure.
- `scripts/deploy/deploy.sh` has a `DEPLOY_DRY_RUN=1` mode that prints each command instead.
- devops.md's secrets table gains `PROMOTION_DEPLOY_KEY`.

### Self-hosting

`docker/selfhost/`: a standalone `compose.yaml` with `postgres` and `server` — image pinned to a
version tag, documents on a local volume (`LP_STORAGE_URL=file:///var/lib/life-pixel`), port 8080
published —, a `.env.example`, and a `README.md`: setup, the reverse proxy for HTTPS, SMTP, the
optional admin console, backups. A self-hosted server never calls our infrastructure.

**Tests and checks**: the `docker` job — hadolint; `docker compose config --quiet` for the local,
deployment and self-hosting files, with example environments; both images built without push —;
the app image started with the local stack (`--profile app`) answering `/healthz` as a non-root
user; `shellcheck` and the dry run of `deploy.sh`; actionlint. The `docker` Dependabot entry.

## H15 — Backups

Production only (`COMPOSE_PROFILES=backup`); staging data is disposable (devops.md). Backups run in
two upstream images, pinned by digest, with the scripts of `scripts/backup/` mounted read-only:
D22's two images stay the only ones built here.

- **Dump** — service `backup-dump`, image `postgres:18-alpine`, whose `pg_dump` matches the server,
  entrypoint `dump-loop.sh`: every day at `BACKUP_TIME` (UTC, `03:15` by default), for each database
  of `BACKUP_DATABASES` (`life_pixel life_pixel_admin`), `pg_dump --format=custom` into the
  `backups` volume as `<database>-<UTC timestamp>.dump.partial`, renamed `.dump` once complete.
- **Upload** — service `backup-upload`, image `rclone/rclone`, entrypoint `upload-loop.sh`: every
  10 minutes, `rclone move` of the finished `*.dump` files to a `crypt` remote over the backup
  bucket in `nl-ams` — encrypted before they leave the host —, with `--no-check-dest`,
  `--s3-no-check-bucket` and `--s3-no-head`, since its credentials can write but neither read nor
  delete. File names stay readable (`filename_encryption = off`), so that a backup is found by date.
- Both services: profile `backup`, the `default` network, 256 MiB, the `backups` volume, their
  loop's process as healthcheck. Each upload logs one JSON line, `backup_uploaded` or
  `backup_failed`, with the file and its size; `infra-vps` alerts when no upload appears for 26
  hours.
- **Variables**: `BACKUP_TIME`, `BACKUP_DATABASES`, `PGHOST`, `PGUSER`, `PGPASSWORD` for the dump;
  for the upload, rclone's `RCLONE_CONFIG_BACKUPS3_*` — type `s3`, provider `Scaleway`, endpoint,
  region, keys — and `RCLONE_CONFIG_BACKUPCRYPT_*` — type `crypt`, remote
  `backups3:<bucket>/<environment>`, `PASSWORD` and `PASSWORD2` in rclone's obscured form. The two
  crypt passwords are the backups' key: the maintainer keeps an offline copy.
- **Retention**: the bucket's lifecycle rule deletes backups after 180 days; the production bucket
  keeps replaced and deleted objects 30 days through versioning (D35) — both set by the maintainer.

### Restore

- `scripts/backup/restore.sh <file> <database-url>`, run by the maintainer with read credentials and
  the crypt passwords: `rclone copy` through the crypt remote, which decrypts, then
  `pg_restore --clean --if-exists --no-owner` into the target database.
- `scripts/backup/rehearse-local.sh` rehearses the chain on the local stack: throwaway crypt
  passwords, one dump and one upload into an S3Mock bucket with the same `rclone` image, a restore
  into a new database, and a comparison of every table's row count with the source.
- The runbook — schedule, location, restore, rehearsal, alert — goes into devops.md's backup
  section. A rehearsal is required before production (devops.md).

**Tests and checks**: `rehearse-local.sh` passes; `shellcheck` on the scripts; `docker compose
config` with the `backup` profile.
