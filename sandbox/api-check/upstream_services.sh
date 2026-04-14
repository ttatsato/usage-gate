#!/bin/bash
# 上流サービス登録
# 使い方: PROJECT_ID=<uuid> ./sandbox/api-check/upstream_services.sh

set -e

: "${PROJECT_ID:?PROJECT_ID is required}"

echo "=== 上流サービス登録 (httpbin) ==="
curl -s -X POST http://localhost:8080/admin/upstream-services \
    -H "Content-Type: application/json" \
    -d "{\"project_id\": \"${PROJECT_ID}\", \"name\": \"httpbin\", \"base_url\": \"https://httpbin.org\"}" | jq

echo ""
echo "=== 上流サービス一覧 ==="
curl -s "http://localhost:8080/admin/upstream-services?project_id=${PROJECT_ID}" | jq
