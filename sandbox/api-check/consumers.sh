#!/bin/bash
# Consumer 作成
# 使い方: PROJECT_ID=<uuid> [PLAN_ID=<uuid>] ./sandbox/api-check/consumers.sh

set -e

: "${PROJECT_ID:?PROJECT_ID is required}"

PLAN_JSON=""
if [ -n "${PLAN_ID}" ]; then
    PLAN_JSON=", \"plan_id\": \"${PLAN_ID}\""
fi

echo "=== Consumer 作成 ==="
curl -s -X POST http://localhost:8080/admin/consumers \
    -H "Content-Type: application/json" \
    -d "{\"project_id\": \"${PROJECT_ID}\", \"external_id\": \"user_12345\"${PLAN_JSON}}" | jq
