#!/bin/sh
# Entrypoint of the backup-upload service (docs/v1/operations.md, "H15 — Backups"): every 10
# minutes, moves each finished dump (*.dump, never *.dump.partial) out of BACKUPS_DIR to the crypt
# remote REMOTE (backupcrypt: by default, whose RCLONE_CONFIG_BACKUPCRYPT_REMOTE names the bucket
# and environment it wraps), with --no-check-dest, --s3-no-check-bucket and --s3-no-head, since
# its credentials can write but neither read nor delete. One rclone invocation per file, so that
# one failure never blocks the others; one JSON line per attempt, backup_uploaded or
# backup_failed, with the file's name and size.
#
# `--once` skips the wait and uploads whatever is ready, a single pass: rehearse-local.sh's mode.
# Runs in the rclone/rclone image: /bin/sh only, no bash.
set -eu

BACKUPS_DIR="${BACKUPS_DIR:-/backups}"
REMOTE="${BACKUP_UPLOAD_REMOTE:-backupcrypt}"
INTERVAL_SECONDS="${BACKUP_UPLOAD_INTERVAL_SECONDS:-600}"

log() { printf 'upload-loop: %s\n' "$*"; }

# One JSON line per upload attempt: event, the file's base name, its size in bytes, the time.
log_json() {
  event="$1"
  file="$2"
  size="$3"
  printf '{"event":"%s","file":"%s","size":%s,"time":"%s"}\n' \
    "$event" "$(basename "$file")" "$size" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

# Moves every finished dump of BACKUPS_DIR to REMOTE, one rclone invocation per file.
upload_once() {
  for file in "$BACKUPS_DIR"/*.dump; do
    [ -e "$file" ] || continue
    size="$(wc -c < "$file" | tr -d ' ')"
    if rclone move "$file" "$REMOTE:$(basename "$file")" \
      --no-check-dest --s3-no-check-bucket --s3-no-head; then
      log_json backup_uploaded "$file" "$size"
    else
      log_json backup_failed "$file" "$size"
    fi
  done
}

main() {
  if [ "${1:-}" = "--once" ]; then
    upload_once
    return
  fi
  log "starting: every ${INTERVAL_SECONDS}s, remote $REMOTE:"
  while true; do
    upload_once
    sleep "$INTERVAL_SECONDS"
  done
}

main "$@"
