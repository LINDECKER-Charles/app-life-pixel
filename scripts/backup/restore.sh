#!/usr/bin/env bash
# Restores one encrypted backup (docs/v1/operations.md, "H15 — Backups"). Run by the maintainer,
# with the read-only rclone credentials and the crypt passwords configured as the rclone remote
# named backupcrypt (or BACKUP_UPLOAD_REMOTE), and libpq's client tools on PATH. `rclone copy`
# fetches and decrypts the file through the crypt remote; `pg_restore` then replays it into the
# target database, dropping what it already holds.
#
# Usage: restore.sh <file> <database-url>
#   <file>          the dump's name on the crypt remote, e.g. life_pixel-20260315T031500Z.dump
#   <database-url>  postgres://user:password@host:port/database to restore into
set -euo pipefail

FILE="${1:?usage: restore.sh <file> <database-url>}"
DATABASE_URL="${2:?usage: restore.sh <file> <database-url>}"
REMOTE="${BACKUP_UPLOAD_REMOTE:-backupcrypt}"

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "restore: fetching and decrypting $FILE from $REMOTE:"
rclone copy "$REMOTE:$FILE" "$WORKDIR"

echo "restore: restoring $FILE into $DATABASE_URL"
pg_restore --clean --if-exists --no-owner --dbname="$DATABASE_URL" "$WORKDIR/$FILE"

echo "restore: done"
