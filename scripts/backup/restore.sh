#!/usr/bin/env bash
# Restores one encrypted backup (docs/v1/operations.md, "H15 — Backups"). Run by the maintainer,
# with the read-only rclone credentials and the crypt passwords configured as the rclone remote
# named backupcrypt (or BACKUP_UPLOAD_REMOTE), and libpq's client tools on PATH. `rclone copy`
# fetches and decrypts the file through the crypt remote; `pg_restore` then replays it into the
# target database, dropping what it already holds.
#
# Usage: restore.sh <file> <database>
#   <file>      the dump's name on the crypt remote, e.g. life_pixel-20260315T031500Z.dump
#   <database>  the database to restore into, on the server libpq's environment names: PGHOST,
#               PGPORT, PGUSER, and the password in PGPASSWORD or a PGPASSFILE — never on the
#               command line, where any process list would show it, nor in this script's output
set -euo pipefail

readonly USAGE="usage: restore.sh <file> <database>"
FILE="${1:?$USAGE}"
DATABASE="${2:?$USAGE}"
REMOTE="${BACKUP_UPLOAD_REMOTE:-backupcrypt}"

if [[ "$DATABASE" == *"://"* || "$DATABASE" == *"="* ]]; then
  echo "restore: give the database's name, not a URL: the password goes in PGPASSWORD or a PGPASSFILE" >&2
  exit 2
fi

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "restore: fetching and decrypting $FILE from $REMOTE:"
rclone copy "$REMOTE:$FILE" "$WORKDIR"

echo "restore: restoring $FILE into the database $DATABASE"
pg_restore --clean --if-exists --no-owner --dbname="$DATABASE" "$WORKDIR/$FILE"

echo "restore: done"
