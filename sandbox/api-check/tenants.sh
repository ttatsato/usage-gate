#!/bin/bash
# テナント作成と一覧取得

set -e

echo "=== テナント作成 ==="
curl -s -X POST http://localhost:8080/admin/tenants \
    -H "Content-Type: application/json" \
    -d '{"name": "acme"}' | jq

echo ""
echo "=== テナント一覧 ==="
curl -s http://localhost:8080/admin/tenants | jq
