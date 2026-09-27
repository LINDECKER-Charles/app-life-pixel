#!/usr/bin/env bash
# Deploys one environment of Life Pixel on the shared VPS, in the six steps of devops.md. It runs
# on the host, where _deploy.yml sends it over SSH with the environment's .env and the job's
# registry token, each in a file of mode 600.
#
#   DEPLOY_SHA                  the commit to run, whose images are tagged :<sha>
#   DEPLOY_PATH                 the environment's folder, /opt/life-pixel-staging or -prod
#   DEPLOY_ENV_FILE             the environment's .env, from ENV_STAGING or ENV_PROD
#   DEPLOY_REGISTRY_USER        the account that logs in to ghcr.io
#   DEPLOY_REGISTRY_TOKEN_FILE  the file holding its token
#   DEPLOY_REPOSITORY           optional, the repository to clone; this project's by default
#   DEPLOY_HEALTH_TIMEOUT       optional, the seconds step 5 waits; 180 by default
#   DEPLOY_DRY_RUN=1            prints each command instead of running it
#
# It never writes to /opt/edge nor restarts the edge, which serves every site of the host.
set -euo pipefail

readonly REGISTRY=ghcr.io
readonly SHARED_NETWORKS=(edge observability)
readonly POLL_SECONDS=5

DRY_RUN="${DEPLOY_DRY_RUN:-0}"
SHA="${DEPLOY_SHA:?DEPLOY_SHA, the commit to deploy}"
TARGET="${DEPLOY_PATH:?DEPLOY_PATH, the folder of the environment}"
ENV_FILE="${DEPLOY_ENV_FILE:?DEPLOY_ENV_FILE, the .env of the environment}"
REGISTRY_USER="${DEPLOY_REGISTRY_USER:?DEPLOY_REGISTRY_USER, the ghcr.io account}"
TOKEN_FILE="${DEPLOY_REGISTRY_TOKEN_FILE:?DEPLOY_REGISTRY_TOKEN_FILE, the file of its token}"
REPOSITORY="${DEPLOY_REPOSITORY:-https://github.com/LINDECKER-Charles/app-life-pixel.git}"
HEALTH_TIMEOUT="${DEPLOY_HEALTH_TIMEOUT:-180}"
COMPOSE_FILES="$TARGET/compose.yaml:$TARGET/compose.deploy.yaml"

log() { printf 'deploy: %s\n' "$*"; }
fail() { printf 'deploy: error: %s\n' "$*" >&2; exit 1; }

# Runs a command, or prints it in a dry run.
run() {
  if [[ "$DRY_RUN" == 1 ]]; then
    printf '+ %s\n' "$*"
  else
    "$@"
  fi
}

# Runs a command with a file on its standard input, or prints both in a dry run.
run_with_input() {
  local input="$1"
  shift
  if [[ "$DRY_RUN" == 1 ]]; then
    printf '+ %s < %s\n' "$*" "$input"
  else
    "$@" < "$input"
  fi
}

compose() {
  run env COMPOSE_FILE="$COMPOSE_FILES" IMAGE_TAG="$SHA" docker compose --project-directory "$TARGET" "$@"
}

# 1. The environment's .env, readable by its owner alone.
write_env() {
  log "1/6 writing $TARGET/.env"
  run mkdir -p "$TARGET"
  run install -m 600 "$ENV_FILE" "$TARGET/.env"
}

# 2. The repository at the promoted commit; the folder may already hold .env, so it is
# initialised in place rather than cloned.
check_out() {
  log "2/6 checking out $SHA"
  if [[ ! -d "$TARGET/.git" ]]; then
    run git -C "$TARGET" init --quiet
    run git -C "$TARGET" remote add origin "$REPOSITORY"
  fi
  run git -C "$TARGET" remote set-url origin "$REPOSITORY"
  run git -C "$TARGET" fetch --quiet --depth 1 origin "$SHA"
  run git -C "$TARGET" checkout --quiet --force --detach "$SHA"
}

# 3. The shared networks infra-vps creates: without them the edge is not there to route.
check_networks() {
  log "3/6 checking the shared networks"
  local network
  for network in "${SHARED_NETWORKS[@]}"; do
    if ! run docker network inspect --format '{{.Name}}' "$network"; then
      fail "the network '$network' does not exist: deploy infra-vps first"
    fi
  done
}

# 4. The images of the commit, then the services on them; orphans of removed services go.
start_services() {
  log "4/6 pulling and starting the images :$SHA"
  run_with_input "$TOKEN_FILE" docker login "$REGISTRY" --username "$REGISTRY_USER" --password-stdin
  compose pull
  compose up -d --remove-orphans
}

# The services of the project that are not healthy, one per line.
unhealthy_services() {
  env COMPOSE_FILE="$COMPOSE_FILES" IMAGE_TAG="$SHA" \
    docker compose --project-directory "$TARGET" ps --all --format '{{.Service}} {{.Health}}' \
    | awk '$2 != "healthy" { print $1 }'
}

# 5. Every service healthy within the timeout, else the logs of those that are not.
wait_healthy() {
  log "5/6 waiting up to ${HEALTH_TIMEOUT}s for every service to be healthy"
  if [[ "$DRY_RUN" == 1 ]]; then
    compose ps --all --format '{{.Service}} {{.Health}}'
    return
  fi
  local deadline=$((SECONDS + HEALTH_TIMEOUT))
  local unhealthy
  while true; do
    unhealthy="$(unhealthy_services)"
    if [[ -z "$unhealthy" ]]; then
      log "every service is healthy"
      return
    fi
    if ((SECONDS >= deadline)); then
      break
    fi
    sleep "$POLL_SECONDS"
  done
  local service
  for service in $unhealthy; do
    log "$service is not healthy; its last logs:"
    compose logs --no-color --tail 200 "$service" || true
  done
  fail "not every service became healthy within ${HEALTH_TIMEOUT}s: $(echo "$unhealthy" | xargs)"
}

# The first domain of CADDY_DOMAINS in the environment's .env.
public_domain() {
  local domains domain
  domains="$(sed -n 's/^CADDY_DOMAINS=//p' "$ENV_FILE" | tail -n 1 | tr ",\"'" ' ')"
  read -r domain _ <<< "$domains" || true
  printf '%s' "${domain:-}"
}

# 6. The public health endpoint through the edge, a warning only: a certificate still being
# issued is diagnosed, never fixed by restarting the edge.
check_public_health() {
  local domain
  domain="$(public_domain)"
  if [[ -z "$domain" ]]; then
    log "6/6 no CADDY_DOMAINS in the .env: skipping the public check"
    return
  fi
  log "6/6 requesting https://$domain/healthz"
  if ! run curl --fail --silent --show-error --max-time 15 --output /dev/null "https://$domain/healthz"; then
    printf '::warning::https://%s/healthz did not answer 200; the edge may still be issuing its certificate\n' "$domain"
  fi
}

main() {
  write_env
  check_out
  check_networks
  start_services
  wait_healthy
  check_public_health
  log "deployed $SHA to $TARGET"
}

main
