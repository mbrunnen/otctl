#!/usr/bin/env bash
# Drives otctl against the mock OTBR server (mock_otbr.py) in this same
# directory. Builds the binary, starts the mock, runs a representative set
# of subcommands, and tears down. Run from the repo root:
#   bash .claude/skills/run-otctl/smoke.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$REPO_ROOT"

PORT="${OTCTL_MOCK_PORT:-8123}"
BASE_URL="http://127.0.0.1:${PORT}"
BIN="target/debug/otctl"

echo "== build =="
cargo build --quiet

echo "== start mock OTBR on :${PORT} =="
python3 "$SCRIPT_DIR/mock_otbr.py" "$PORT" > /tmp/mock_otbr.log 2>&1 &
MOCK_PID=$!
trap 'kill "$MOCK_PID" 2>/dev/null || true' EXIT

timeout 5 bash -c "until grep -q MOCK-OTBR-READY /tmp/mock_otbr.log 2>/dev/null; do sleep 0.1; done"

run() {
  echo "+ otctl $*"
  "$BIN" --base-url "$BASE_URL" "$@"
  echo
}

run node info
run node state
run --json node state
run dataset active
run devices list --no-refresh
run diagnostics list

echo "+ otctl actions submit --file - (stdin)"
echo '{"data":[]}' | "$BIN" --base-url "$BASE_URL" actions submit --file -
echo

echo "+ otctl node reset --yes"
"$BIN" --base-url "$BASE_URL" node reset --yes
echo

echo "== all commands succeeded =="
