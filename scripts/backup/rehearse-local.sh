#!/usr/bin/env bash
# Rehearses H15's backup chain against the local stack (docs/v1/operations.md, "H15 — Backups",
# and its "Restore" section): dump-loop.sh dumps one database, upload-loop.sh moves it, encrypted,
# to a throwaway bucket on the local S3Mock, the same rclone image then fetches and decrypts it,
# and pg_restore replays it into a new database, whose every table's row count is then compared
# with the source. Crypt passwords are generated fresh for the run.
#
# Creates only two resources, both named at random: a Postgres database and an S3Mock bucket. Run
# `main`, then `cleanup` (a trap, so it also runs on failure) drops the database and purges the
# bucket, so nothing of this run outlives it.
#
# Run from the repository root, with the local stack up (`docker compose up -d --wait`) and the
# `.env` it started from. `PGPASSWORD` is read from the environment, or from `.env`'s
# LP_POSTGRES_PASSWORD if `.env` is present in the current directory.
set -euo pipefail

# The exact images of compose.yaml (postgres, so `pg_dump`/`pg_restore` match the server) and of
# compose.deploy.yaml's backup-upload service (rclone, so the rehearsal proves what production
# runs), pinned by digest.
POSTGRES_IMAGE="postgres:18-alpine@sha256:77f585114c32fbca283dc835b0596f4e52b51b4c6662d7810b2f4084f60a1873"
RCLONE_IMAGE="rclone/rclone@sha256:45401ad7410db1d67ffdb58e19059ad20b0d8e0285a60e38bbec55cc1019c7a5"

NETWORK="${REHEARSE_NETWORK:-life-pixel_default}"
SOURCE_DATABASE="${REHEARSE_SOURCE_DATABASE:-life_pixel}"
PGHOST="${REHEARSE_PGHOST:-postgres}"
PGUSER="${REHEARSE_PGUSER:-postgres}"
S3_ENDPOINT="${REHEARSE_S3_ENDPOINT:-http://objectstore:9090}"
S3_ACCESS_KEY_ID="${REHEARSE_S3_ACCESS_KEY_ID:-local}"
S3_SECRET_ACCESS_KEY="${REHEARSE_S3_SECRET_ACCESS_KEY:-local}"

if [[ -z "${PGPASSWORD:-}" && -f .env ]]; then
  PGPASSWORD="$(sed -n 's/^LP_POSTGRES_PASSWORD=//p' .env | tail -n 1)"
fi
PGPASSWORD="${PGPASSWORD:?PGPASSWORD, or LP_POSTGRES_PASSWORD in ./.env: the local stack postgres superuser password}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STAMP="$(date -u +%Y%m%d%H%M%S)$$"
RESTORE_DATABASE="rehearse_restore_$STAMP"
BUCKET="rehearse-backups-$STAMP"
DUMP_DIR="$(mktemp -d)"
DOWNLOAD_DIR="$(mktemp -d)"
CRYPT_PASSWORD=""
CRYPT_PASSWORD2=""
BUCKET_CREATED=0
DATABASE_CREATED=0

log() { printf 'rehearse-local: %s\n' "$*"; }
fail() { printf 'rehearse-local: error: %s\n' "$*" >&2; exit 1; }

# A throwaway Postgres client container on the stack's network, authenticated as PGUSER.
pg() {
  docker run --rm --network "$NETWORK" -e PGPASSWORD="$PGPASSWORD" "$@"
}

# A throwaway rclone container on the stack's network, configured with the S3Mock remote
# (backups3:) and a crypt remote (backupcrypt:) wrapping this run's throwaway bucket.
rclone_run() {
  docker run --rm --network "$NETWORK" \
    -e RCLONE_CONFIG_BACKUPS3_TYPE=s3 \
    -e RCLONE_CONFIG_BACKUPS3_PROVIDER=Other \
    -e RCLONE_CONFIG_BACKUPS3_ENV_AUTH=false \
    -e RCLONE_CONFIG_BACKUPS3_ENDPOINT="$S3_ENDPOINT" \
    -e RCLONE_CONFIG_BACKUPS3_REGION=us-east-1 \
    -e RCLONE_CONFIG_BACKUPS3_ACCESS_KEY_ID="$S3_ACCESS_KEY_ID" \
    -e RCLONE_CONFIG_BACKUPS3_SECRET_ACCESS_KEY="$S3_SECRET_ACCESS_KEY" \
    -e RCLONE_CONFIG_BACKUPS3_FORCE_PATH_STYLE=true \
    -e RCLONE_CONFIG_BACKUPCRYPT_TYPE=crypt \
    -e RCLONE_CONFIG_BACKUPCRYPT_REMOTE="backups3:$BUCKET" \
    -e RCLONE_CONFIG_BACKUPCRYPT_FILENAME_ENCRYPTION=off \
    -e RCLONE_CONFIG_BACKUPCRYPT_PASSWORD="$CRYPT_PASSWORD" \
    -e RCLONE_CONFIG_BACKUPCRYPT_PASSWORD2="$CRYPT_PASSWORD2" \
    "$@"
}

# rclone's own light obfuscation, required of RCLONE_CONFIG_*_PASSWORD[2]: a plain password is
# rejected at connection time ("is it obscured?").
obscure() {
  docker run --rm --entrypoint rclone "$RCLONE_IMAGE" obscure "$1"
}

# One line per table of DATABASE, "<table>|<row count>", sorted by table name: built as a single
# generated UNION ALL query, so that one round trip compares every table at once, and a table
# present on one side alone shows up as a diff too.
table_row_counts() {
  local database="$1" query
  query="$(pg "$POSTGRES_IMAGE" psql -h "$PGHOST" -U "$PGUSER" -d "$database" -tAc "
    SELECT coalesce(string_agg(
      format('SELECT %L::text AS t, count(*)::bigint AS n FROM %I', tablename, tablename),
      ' UNION ALL '
    ), 'SELECT NULL::text AS t, NULL::bigint AS n WHERE false')
    FROM pg_tables WHERE schemaname = 'public';
  ")"
  pg "$POSTGRES_IMAGE" psql -h "$PGHOST" -U "$PGUSER" -d "$database" -tAF'|' -c "$query ORDER BY 1;"
}

cleanup() {
  local status=$?
  if ((DATABASE_CREATED)); then
    log "dropping $RESTORE_DATABASE"
    pg "$POSTGRES_IMAGE" dropdb -h "$PGHOST" -U "$PGUSER" --if-exists "$RESTORE_DATABASE" || true
  fi
  if ((BUCKET_CREATED)); then
    log "purging bucket $BUCKET"
    rclone_run "$RCLONE_IMAGE" purge "backups3:$BUCKET" || true
  fi
  rm -rf "$DUMP_DIR" "$DOWNLOAD_DIR"
  exit "$status"
}
trap cleanup EXIT

main() {
  log "source database: $SOURCE_DATABASE, restore database: $RESTORE_DATABASE, bucket: $BUCKET"

  CRYPT_PASSWORD="$(obscure "$(openssl rand -hex 32)")"
  CRYPT_PASSWORD2="$(obscure "$(openssl rand -hex 32)")"

  log "creating bucket $BUCKET on S3Mock"
  rclone_run "$RCLONE_IMAGE" mkdir "backups3:$BUCKET"
  BUCKET_CREATED=1

  log "dumping $SOURCE_DATABASE (dump-loop.sh --once)"
  pg -v "$SCRIPT_DIR:/scripts:ro" -v "$DUMP_DIR:/backups" \
    -e BACKUP_DATABASES="$SOURCE_DATABASE" -e PGHOST="$PGHOST" -e PGUSER="$PGUSER" \
    --entrypoint bash "$POSTGRES_IMAGE" /scripts/dump-loop.sh --once

  local dump_file dump_name
  dump_file="$(find "$DUMP_DIR" -maxdepth 1 -type f -name '*.dump' | head -n 1)"
  [[ -n "$dump_file" ]] || fail "dump-loop.sh produced no .dump file"
  dump_name="$(basename "$dump_file")"
  log "dumped $dump_name"

  log "uploading $dump_name (upload-loop.sh --once)"
  rclone_run -v "$SCRIPT_DIR:/scripts:ro" -v "$DUMP_DIR:/backups" \
    --entrypoint sh "$RCLONE_IMAGE" /scripts/upload-loop.sh --once

  [[ -z "$(find "$DUMP_DIR" -maxdepth 1 -type f -name '*.dump')" ]] \
    || fail "$dump_name is still local after upload-loop.sh: see its backup_failed line above"

  log "fetching and decrypting $dump_name"
  rclone_run -v "$DOWNLOAD_DIR:/download" "$RCLONE_IMAGE" \
    copy "backupcrypt:$dump_name" /download

  log "counting rows of $SOURCE_DATABASE"
  local source_counts
  source_counts="$(table_row_counts "$SOURCE_DATABASE")"

  log "creating $RESTORE_DATABASE"
  pg "$POSTGRES_IMAGE" createdb -h "$PGHOST" -U "$PGUSER" "$RESTORE_DATABASE"
  DATABASE_CREATED=1

  log "restoring $dump_name into $RESTORE_DATABASE"
  pg -v "$DOWNLOAD_DIR:/download:ro" "$POSTGRES_IMAGE" \
    pg_restore --clean --if-exists --no-owner -h "$PGHOST" -U "$PGUSER" \
    -d "$RESTORE_DATABASE" "/download/$dump_name"

  log "counting rows of $RESTORE_DATABASE"
  local restore_counts
  restore_counts="$(table_row_counts "$RESTORE_DATABASE")"

  if [[ "$source_counts" == "$restore_counts" ]]; then
    log "row counts match for every table of $SOURCE_DATABASE"
  else
    printf 'rehearse-local: row counts differ:\n--- %s\n+++ %s\n' "$SOURCE_DATABASE" "$RESTORE_DATABASE" >&2
    diff <(printf '%s\n' "$source_counts") <(printf '%s\n' "$restore_counts") >&2 || true
    fail "the restored database does not match the source"
  fi
}

main
