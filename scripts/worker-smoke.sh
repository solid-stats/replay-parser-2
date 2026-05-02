#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.worker-smoke.yml}"
COMPOSE_PROJECT="${COMPOSE_PROJECT:-replay-parser-worker-smoke}"

export RABBITMQ_AMQP_PORT="${RABBITMQ_AMQP_PORT:-56729}"
export RABBITMQ_MANAGEMENT_PORT="${RABBITMQ_MANAGEMENT_PORT:-15679}"
export MINIO_API_PORT="${MINIO_API_PORT:-19000}"
export MINIO_CONSOLE_PORT="${MINIO_CONSOLE_PORT:-19001}"
export MINIO_ROOT_USER="${MINIO_ROOT_USER:-minioadmin}"
export MINIO_ROOT_PASSWORD="${MINIO_ROOT_PASSWORD:-minioadmin}"
export MINIO_BUCKET="${MINIO_BUCKET:-replay-parser-smoke}"

compose() {
  docker compose -p "$COMPOSE_PROJECT" -f "$ROOT_DIR/$COMPOSE_FILE" "$@"
}

wait_for_url() {
  local url="$1"
  local auth="${2:-}"
  for _ in $(seq 1 90); do
    if [ -n "$auth" ]; then
      if curl --fail --silent --show-error --user "$auth" "$url" >/dev/null 2>&1; then
        return 0
      fi
    elif curl --fail --silent --show-error "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "Timed out waiting for $url" >&2
  return 1
}

cleanup() {
  if [ "${KEEP_SMOKE_INFRA:-0}" != "1" ]; then
    compose down -v --remove-orphans >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT

command -v docker >/dev/null || { echo "docker is required" >&2; exit 1; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }

cd "$ROOT_DIR"

compose up -d rabbitmq minio
wait_for_url "http://127.0.0.1:${RABBITMQ_MANAGEMENT_PORT}/api/overview" "guest:guest"
wait_for_url "http://127.0.0.1:${MINIO_API_PORT}/minio/health/ready"

export REPLAY_PARSER_LIVE_SMOKE=1
export REPLAY_PARSER_AMQP_URL="amqp://guest:guest@127.0.0.1:${RABBITMQ_AMQP_PORT}/%2f"
export REPLAY_PARSER_JOB_QUEUE="${REPLAY_PARSER_JOB_QUEUE:-parse.jobs}"
export REPLAY_PARSER_RESULT_EXCHANGE="${REPLAY_PARSER_RESULT_EXCHANGE:-parse.results}"
export REPLAY_PARSER_COMPLETED_ROUTING_KEY="${REPLAY_PARSER_COMPLETED_ROUTING_KEY:-parse.completed}"
export REPLAY_PARSER_FAILED_ROUTING_KEY="${REPLAY_PARSER_FAILED_ROUTING_KEY:-parse.failed}"
export REPLAY_PARSER_S3_BUCKET="$MINIO_BUCKET"
export AWS_REGION="${AWS_REGION:-us-east-1}"
export REPLAY_PARSER_S3_ENDPOINT="http://127.0.0.1:${MINIO_API_PORT}"
export REPLAY_PARSER_S3_FORCE_PATH_STYLE=true
export AWS_ACCESS_KEY_ID="$MINIO_ROOT_USER"
export AWS_SECRET_ACCESS_KEY="$MINIO_ROOT_PASSWORD"

cargo test -p parser-worker --test live_smoke -- --ignored --nocapture
