#!/bin/bash
# 使用量取得
# 使い方: TENANT_ID=<uuid> ./sandbox/usage.sh

set -e

: "${TENANT_ID:?TENANT_ID is required. Usage: TENANT_ID=<uuid> ./sandbox/usage.sh}"

echo "=== 使用量取得 ==="
curl -s "http://localhost:8080/admin/usage?tenant_id=${TENANT_ID}" | jq

echo ""
echo "=== 期間指定 (2026-04-01 以降) ==="
curl -s "http://localhost:8080/admin/usage?tenant_id=${TENANT_ID}&start_date=2026-04-01T00:00:00Z" | jq
