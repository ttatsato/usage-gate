#!/bin/bash
# Proxy エンドポイントのテスト
# 使い方: API_KEY=<key> ./sandbox/proxy.sh

set -e

: "${API_KEY:?API_KEY is required. Usage: API_KEY=<key> ./sandbox/proxy.sh}"

echo "=== 認証なし (期待: 401) ==="
curl -s -w "\nHTTP %{http_code}\n" http://localhost:8080/proxy/test

echo ""
echo "=== 無効なキー (期待: 401) ==="
curl -s -w "\nHTTP %{http_code}\n" -H "x-api-key: invalid-key" http://localhost:8080/proxy/test

echo ""
echo "=== 有効なキー (期待: 200) ==="
curl -s -w "\nHTTP %{http_code}\n" -H "x-api-key: ${API_KEY}" http://localhost:8080/proxy/test
