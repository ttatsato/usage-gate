#!/bin/bash
# Consumer 作成
# 使い方: TENANT_ID=<uuid> ./sandbox/consumers.sh

set -e

: "${TENANT_ID:?TENANT_ID is required. Usage: TENANT_ID=<uuid> ./sandbox/consumers.sh}"

echo "=== Consumer 作成 ==="
curl -s -X POST http://localhost:8080/admin/consumers \
    -H "Content-Type: application/json" \
    -d "{\"tenant_id\": \"${TENANT_ID}\", \"external_id\": \"user_12345\"}" | jq
