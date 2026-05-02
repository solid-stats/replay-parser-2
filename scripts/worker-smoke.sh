#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.worker-smoke.yml}"
COMPOSE_PROJECT="${COMPOSE_PROJECT:-replay-parser-worker-smoke}"

export WORKER_IMAGE="${WORKER_IMAGE:-replay-parser-2-worker-smoke:latest}"
export RABBITMQ_AMQP_PORT="${RABBITMQ_AMQP_PORT:-56729}"
export RABBITMQ_MANAGEMENT_PORT="${RABBITMQ_MANAGEMENT_PORT:-15679}"
export RABBITMQ_USER="${RABBITMQ_USER:-guest}"
export RABBITMQ_PASSWORD="${RABBITMQ_PASSWORD:-guest}"
export MINIO_API_PORT="${MINIO_API_PORT:-19000}"
export MINIO_CONSOLE_PORT="${MINIO_CONSOLE_PORT:-19001}"
export MINIO_ROOT_USER="${MINIO_ROOT_USER:-minioadmin}"
export MINIO_ROOT_PASSWORD="${MINIO_ROOT_PASSWORD:-minioadmin}"
export MINIO_BUCKET="${MINIO_BUCKET:-replay-parser-smoke}"
export WORKER_A_PROBE_PORT="${WORKER_A_PROBE_PORT:-18081}"
export WORKER_B_PROBE_PORT="${WORKER_B_PROBE_PORT:-18082}"

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

build_amqp_url() {
  local host="$1"
  local port="$2"
  printf '%s://%s:%s@%s:%s/%%2f' "amqp" "$RABBITMQ_USER" "$RABBITMQ_PASSWORD" "$host" "$port"
}

rabbitmq_api() {
  local method="$1"
  local path="$2"
  local body="$3"
  curl --fail --silent --show-error \
    --user "${RABBITMQ_USER}:${RABBITMQ_PASSWORD}" \
    --header "content-type: application/json" \
    --request "$method" \
    --data "$body" \
    "http://127.0.0.1:${RABBITMQ_MANAGEMENT_PORT}${path}" >/dev/null
}

declare_broker_topology() {
  local vhost="%2F"
  local completed_queue="parse.completed.smoke"
  local failed_queue="parse.failed.smoke"
  rabbitmq_api PUT "/api/queues/${vhost}/${REPLAY_PARSER_JOB_QUEUE}" \
    '{"durable":true,"auto_delete":false,"arguments":{}}'
  rabbitmq_api PUT "/api/queues/${vhost}/${completed_queue}" \
    '{"durable":true,"auto_delete":false,"arguments":{}}'
  rabbitmq_api PUT "/api/queues/${vhost}/${failed_queue}" \
    '{"durable":true,"auto_delete":false,"arguments":{}}'
  rabbitmq_api PUT "/api/exchanges/${vhost}/${REPLAY_PARSER_RESULT_EXCHANGE}" \
    '{"type":"direct","durable":true,"auto_delete":false,"internal":false,"arguments":{}}'
  rabbitmq_api POST "/api/bindings/${vhost}/e/${REPLAY_PARSER_RESULT_EXCHANGE}/q/${completed_queue}" \
    "$(printf '{"routing_key":"%s","arguments":{}}' "$REPLAY_PARSER_COMPLETED_ROUTING_KEY")"
  rabbitmq_api POST "/api/bindings/${vhost}/e/${REPLAY_PARSER_RESULT_EXCHANGE}/q/${failed_queue}" \
    "$(printf '{"routing_key":"%s","arguments":{}}' "$REPLAY_PARSER_FAILED_ROUTING_KEY")"
}

assert_log_contains() {
  local pattern="$1"
  local label="$2"
  local logs_file="$3"
  if ! grep -q "$pattern" "$logs_file"; then
    echo "missing smoke log: ${label}" >&2
    tail -200 "$logs_file" >&2 || true
    return 1
  fi
}

run_timeweb_s3_smoke() {
  for required in AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY REPLAY_PARSER_S3_BUCKET REPLAY_PARSER_S3_ENDPOINT; do
    if [ -z "${!required:-}" ]; then
      echo "TIMEWEB_S3_SMOKE requires ${required}" >&2
      exit 2
    fi
  done

  if ! command -v aws >/dev/null; then
    echo "timeweb_conditional_write=failed"
    echo "aws CLI is required for TIMEWEB_S3_SMOKE" >&2
    exit 127
  fi

  export AWS_REGION="${AWS_REGION:-ru-1}"
  local key="artifacts/phase-07-timeweb-smoke/$(date +%s)-$$.json"
  local body
  local error_log
  body="$(mktemp)"
  error_log="$(mktemp)"
  printf '{"phase":"07","smoke":"timeweb"}\n' >"$body"

  local status=0
  if aws --no-cli-pager --endpoint-url "$REPLAY_PARSER_S3_ENDPOINT" s3api put-object \
    --bucket "$REPLAY_PARSER_S3_BUCKET" \
    --key "$key" \
    --body "$body" \
    --if-none-match '*' >/dev/null 2>"$error_log"; then
    if aws --no-cli-pager --endpoint-url "$REPLAY_PARSER_S3_ENDPOINT" s3api put-object \
      --bucket "$REPLAY_PARSER_S3_BUCKET" \
      --key "$key" \
      --body "$body" \
      --if-none-match '*' >/dev/null 2>"$error_log"; then
      echo "timeweb_conditional_write=failed"
      status=1
    else
      echo "timeweb_conditional_write=pass"
    fi
    aws --no-cli-pager --endpoint-url "$REPLAY_PARSER_S3_ENDPOINT" s3api delete-object \
      --bucket "$REPLAY_PARSER_S3_BUCKET" \
      --key "$key" >/dev/null 2>&1 || true
  elif grep -Eiq "not.?implemented|not supported|unsupported|invalidrequest|precondition" "$error_log"; then
    echo "timeweb_conditional_write=unsupported_fallback_required"
  else
    echo "timeweb_conditional_write=failed"
    status=1
  fi

  rm -f "$body" "$error_log"
  exit "$status"
}

cleanup() {
  if [ "${KEEP_SMOKE_INFRA:-0}" != "1" ]; then
    compose down -v --remove-orphans >/dev/null 2>&1 || true
  fi
}

if [ "${TIMEWEB_S3_SMOKE:-0}" = "1" ]; then
  run_timeweb_s3_smoke
fi

command -v docker >/dev/null || { echo "docker is required" >&2; exit 1; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }

trap cleanup EXIT
cd "$ROOT_DIR"

export REPLAY_PARSER_JOB_QUEUE="${REPLAY_PARSER_JOB_QUEUE:-parse.jobs}"
export REPLAY_PARSER_RESULT_EXCHANGE="${REPLAY_PARSER_RESULT_EXCHANGE:-parse.results}"
export REPLAY_PARSER_COMPLETED_ROUTING_KEY="${REPLAY_PARSER_COMPLETED_ROUTING_KEY:-parse.completed}"
export REPLAY_PARSER_FAILED_ROUTING_KEY="${REPLAY_PARSER_FAILED_ROUTING_KEY:-parse.failed}"
export REPLAY_PARSER_AMQP_URL
REPLAY_PARSER_AMQP_URL="$(build_amqp_url "127.0.0.1" "$RABBITMQ_AMQP_PORT")"
export REPLAY_PARSER_CONTAINER_AMQP_URL
REPLAY_PARSER_CONTAINER_AMQP_URL="$(build_amqp_url "rabbitmq" "5672")"
export REPLAY_PARSER_LIVE_SMOKE=1
export REPLAY_PARSER_CONTAINER_SMOKE=1
export REPLAY_PARSER_S3_BUCKET="$MINIO_BUCKET"
export AWS_REGION="${AWS_REGION:-us-east-1}"
export REPLAY_PARSER_S3_ENDPOINT="http://127.0.0.1:${MINIO_API_PORT}"
export REPLAY_PARSER_S3_FORCE_PATH_STYLE=true
export AWS_ACCESS_KEY_ID="$MINIO_ROOT_USER"
printf -v AWS_SECRET_ACCESS_KEY '%s' "$MINIO_ROOT_PASSWORD"
export AWS_SECRET_ACCESS_KEY

docker build -t "$WORKER_IMAGE" .
compose up -d rabbitmq minio
wait_for_url "http://127.0.0.1:${RABBITMQ_MANAGEMENT_PORT}/api/overview" "${RABBITMQ_USER}:${RABBITMQ_PASSWORD}"
wait_for_url "http://127.0.0.1:${MINIO_API_PORT}/minio/health/ready"
declare_broker_topology

export REPLAY_PARSER_CONTAINER_SMOKE_SETUP_ONLY=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p parser-worker --test live_smoke -- --ignored --nocapture
unset REPLAY_PARSER_CONTAINER_SMOKE_SETUP_ONLY

compose up -d worker-a worker-b
wait_for_url "http://127.0.0.1:${WORKER_A_PROBE_PORT}/livez"
wait_for_url "http://127.0.0.1:${WORKER_A_PROBE_PORT}/readyz"
wait_for_url "http://127.0.0.1:${WORKER_B_PROBE_PORT}/livez"
wait_for_url "http://127.0.0.1:${WORKER_B_PROBE_PORT}/readyz"

test_status=0
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p parser-worker --test live_smoke -- --ignored --nocapture || test_status=$?

logs_file="$(mktemp)"
compose logs --no-color worker-a worker-b >"$logs_file" || true
if [ "$test_status" -ne 0 ]; then
  cat "$logs_file" >&2
  rm -f "$logs_file"
  exit "$test_status"
fi

assert_log_contains '"worker_id":"worker-a' "worker-a worker_id" "$logs_file"
assert_log_contains '"worker_id":"worker-b' "worker-b worker_id" "$logs_file"
assert_log_contains 'worker_job_received' "worker_job_received event" "$logs_file"
assert_log_contains 'worker_artifact_reused' "worker_artifact_reused event" "$logs_file"
assert_log_contains 'worker_result_published' "worker_result_published event" "$logs_file"
rm -f "$logs_file"
