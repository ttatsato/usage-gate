#!/bin/bash
# API キー作成と一覧
# 使い方: CONSUMER_ID=<uuid> ./sandbox/api_keys.sh

set -e

: "${CONSUMER_ID:?CONSUMER_ID is required. Usage: CONSUMER_ID=<uuid> ./sandbox/api_keys.sh}"

echo "=== API キー作成 ==="
curl -s -X POST http://localhost:8080/admin/api-keys \
    -H "Content-Type: application/json" \
    -d "{\"consumer_id\": \"${CONSUMER_ID}\", \"name\": \"my-key\"}" | jq

echo ""
echo "=== API キー一覧 ==="
curl -s http://localhost:8080/admin/api-keys | jq

echo ""
echo "=== 存在しない consumer_id で作成 (期待: 404) ==="
curl -s -w "\nHTTP %{http_code}\n" -X POST http://localhost:8080/admin/api-keys \
    -H "Content-Type: application/json" \
    -d '{"consumer_id": "00000000-0000-0000-0000-000000000000", "name": "test"}'
