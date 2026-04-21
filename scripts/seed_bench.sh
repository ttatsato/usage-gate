#!/usr/bin/env bash
# Seed data for k6 benchmark runs.
# Prereq: usage-gate API running on $API_URL, mock upstream on $UPSTREAM_URL.
# Output: tests/k6/.env with API keys for k6 to consume.
set -euo pipefail

API_URL="${API_URL:-http://localhost:8080}"
UPSTREAM_URL="${UPSTREAM_URL:-http://localhost:9090}"
OUT="${OUT:-tests/k6/.env}"

need() { command -v "$1" >/dev/null || { echo "missing: $1" >&2; exit 1; }; }
need curl
need jq

post() { curl -sS -X POST "$API_URL$1" -H "content-type: application/json" -d "$2"; }

echo "→ tenant"
TENANT_ID=$(post /admin/tenants '{"name":"bench"}' | jq -r .id)

echo "→ project"
PROJECT_ID=$(post /admin/projects "{\"tenant_id\":\"$TENANT_ID\",\"name\":\"bench\"}" | jq -r .id)

echo "→ plan (no quota)"
PLAN_NOQUOTA_ID=$(post /admin/plans "{\"project_id\":\"$PROJECT_ID\",\"name\":\"noquota\"}" | jq -r .id)

echo "→ plan (with quota)"
PLAN_QUOTA_ID=$(post /admin/plans "{\"project_id\":\"$PROJECT_ID\",\"name\":\"quota\",\"monthly_request_quota\":100000000,\"daily_request_quota\":100000000,\"hourly_request_quota\":100000000,\"per_second_request_limit\":100000}" | jq -r .id)

echo "→ consumer (noquota)"
CONSUMER_NOQUOTA_ID=$(post /admin/consumers "{\"project_id\":\"$PROJECT_ID\",\"plan_id\":\"$PLAN_NOQUOTA_ID\"}" | jq -r .id)

echo "→ consumer (quota)"
CONSUMER_QUOTA_ID=$(post /admin/consumers "{\"project_id\":\"$PROJECT_ID\",\"plan_id\":\"$PLAN_QUOTA_ID\"}" | jq -r .id)

echo "→ api_key (noquota)"
API_KEY_NOQUOTA=$(post /admin/api-keys "{\"consumer_id\":\"$CONSUMER_NOQUOTA_ID\",\"name\":\"noquota\"}" | jq -r .key)

echo "→ api_key (quota)"
API_KEY_QUOTA=$(post /admin/api-keys "{\"consumer_id\":\"$CONSUMER_QUOTA_ID\",\"name\":\"quota\"}" | jq -r .key)

echo "→ upstream service (mock)"
post /admin/upstream-services "{\"project_id\":\"$PROJECT_ID\",\"name\":\"mock\",\"base_url\":\"$UPSTREAM_URL\"}" >/dev/null

cat > "$OUT" <<EOF
API_URL=$API_URL
API_KEY_NOQUOTA=$API_KEY_NOQUOTA
API_KEY_QUOTA=$API_KEY_QUOTA
TENANT_ID=$TENANT_ID
PROJECT_ID=$PROJECT_ID
EOF

echo
echo "done → $OUT"
cat "$OUT"
