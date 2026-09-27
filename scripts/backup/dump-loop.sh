#!/usr/bin/env bash
# Entrypoint of the backup-dump service (docs/v1/operations.md, "H15 — Backups"): every day at
# BACKUP_TIME (UTC), dumps each database of BACKUP_DATABASES with `pg_dump --format=custom` into
# BACKUPS_DIR, first as <database>-<UTC timestamp>.dump.partial, renamed .dump only once the dump
# has completed. `pg_dump` reads PGHOST, PGUSER and PGPASSWORD itself; nothing else is needed.
#
# `--once` skips the daily wait and dumps immediately, a single time: rehearse-local.sh's mode.
set -euo pipefail

BACKUP_TIME="${BACKUP_TIME:-03:15}"
BACKUP_DATABASES="${BACKUP_DATABASES:?BACKUP_DATABASES, the space-separated databases to dump}"
BACKUPS_DIR="${BACKUPS_DIR:-/backups}"

log() { printf 'dump-loop: %s\n' "$*"; }

# Dumps every database of BACKUP_DATABASES into BACKUPS_DIR, each first as a .dump.partial file,
# renamed .dump only once pg_dump has exited successfully: a reader never sees a half-written dump
# under the final name.
dump_once() {
  local timestamp database partial final
  timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
  for database in $BACKUP_DATABASES; do
    partial="$BACKUPS_DIR/$database-$timestamp.dump.partial"
    final="$BACKUPS_DIR/$database-$timestamp.dump"
    log "dumping $database into $partial"
    pg_dump --format=custom --file="$partial" "$database"
    mv "$partial" "$final"
    log "wrote $final"
  done
}

# The epoch of the next occurrence of BACKUP_TIME (UTC): today if still ahead, tomorrow otherwise.
next_run_epoch() {
  local now target
  now="$(date -u +%s)"
  target="$(date -u -d "$BACKUP_TIME" +%s)"
  if ((target <= now)); then
    target=$((target + 86400))
  fi
  printf '%s' "$target"
}

main() {
  mkdir -p "$BACKUPS_DIR"
  if [[ "${1:-}" == "--once" ]]; then
    dump_once
    return
  fi
  log "starting: daily at $BACKUP_TIME UTC, databases: $BACKUP_DATABASES"
  while true; do
    local target now
    target="$(next_run_epoch)"
    now="$(date -u +%s)"
    log "sleeping $((target - now))s until $(date -u -d "@$target" +%Y-%m-%dT%H:%M:%SZ)"
    sleep "$((target - now))"
    dump_once || log "dump failed, retrying tomorrow"
  done
}

main "$@"
