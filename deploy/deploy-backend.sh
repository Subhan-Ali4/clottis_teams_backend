#!/usr/bin/env bash
set -Eeuo pipefail
IMAGE="${1:?image tag required}"
cd /home/dev/clottis-teams/deployment
docker image inspect "$IMAGE" >/dev/null
OLD="$(docker inspect --format '{{.Config.Image}}' clottis-teams-backend-1)"
OVERRIDE=backend-image.override.yml
printf 'services:\n  backend:\n    image: "%s"\n' "$IMAGE" > "$OVERRIDE"
chmod 600 "$OVERRIDE"
dc() { docker compose -p clottis-teams --env-file .env -f compose.yml -f "$OVERRIDE" "$@"; }
dc config --quiet
dc up -d --no-deps backend
for attempt in $(seq 1 20); do
  if curl -fsS --max-time 4 http://127.0.0.1:8082/api/v1/health/ready > /dev/null; then exit 0; fi
  sleep 3
done
printf 'services:\n  backend:\n    image: "%s"\n' "$OLD" > "$OVERRIDE"
dc up -d --no-deps backend
echo "Backend readiness failed; reverted to previous image" >&2
exit 1
