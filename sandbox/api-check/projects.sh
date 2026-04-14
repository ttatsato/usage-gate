#!/bin/bash
# プロジェクト作成
# 使い方: TENANT_ID=<uuid> ./sandbox/api-check/projects.sh

set -e

: "${TENANT_ID:?TENANT_ID is required}"

echo "=== プロジェクト作成 ==="
curl -s -X POST http://localhost:8080/admin/projects \
    -H "Content-Type: application/json" \
    -d "{\"tenant_id\": \"${TENANT_ID}\", \"name\": \"default\"}" | jq

echo ""
echo "=== プロジェクト一覧 ==="
curl -s "http://localhost:8080/admin/projects?tenant_id=${TENANT_ID}" | jq
