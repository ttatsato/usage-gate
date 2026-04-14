#!/bin/bash
# プラン作成
# 使い方: PROJECT_ID=<uuid> ./sandbox/api-check/plans.sh

set -e

: "${PROJECT_ID:?PROJECT_ID is required}"

echo "=== プラン作成（Free: 100回/月） ==="
curl -s -X POST http://localhost:8080/admin/plans \
    -H "Content-Type: application/json" \
    -d "{\"project_id\": \"${PROJECT_ID}\", \"name\": \"free\", \"monthly_request_quota\": 100}" | jq

echo ""
echo "=== プラン一覧 ==="
curl -s "http://localhost:8080/admin/plans?project_id=${PROJECT_ID}" | jq
