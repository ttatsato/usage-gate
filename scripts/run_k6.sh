#!/usr/bin/env bash
# Run a k6 scenario. Filename encodes the test identity: <scenario>_<auth>.json.
# Re-running overwrites. Which codebase state produced the numbers is tracked by git.
# Usage: ./scripts/run_k6.sh <smoke|load|stress> <quota|noquota>
set -euo pipefail

SCENARIO="${1:?scenario required: smoke | load | stress}"
AUTH="${2:?auth required: quota | noquota}"

SCRIPT="tests/k6/${SCENARIO}.js"
[[ -f "$SCRIPT" ]] || { echo "no such scenario: $SCRIPT" >&2; exit 2; }

# shellcheck disable=SC1091
source tests/k6/.env

case "$AUTH" in
  quota)   API_KEY="$API_KEY_QUOTA" ;;
  noquota) API_KEY="$API_KEY_NOQUOTA" ;;
  *) echo "auth must be quota or noquota" >&2; exit 2 ;;
esac

OUT_DIR="tests/k6/results"
mkdir -p "$OUT_DIR"
SUMMARY="$OUT_DIR/${SCENARIO}_${AUTH}.json"

API_URL="$API_URL" API_KEY="$API_KEY" \
  k6 run --summary-export="$SUMMARY" "$SCRIPT"

echo
echo "summary → $SUMMARY"
