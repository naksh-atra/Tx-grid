#!/usr/bin/env bash
set -euo pipefail

# End-to-end test script
# Creates a tmux test session, runs the binary, verifies output

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="${REPO_DIR}/target/release/tmux-taskgrid"

echo "=== tmux-taskgrid E2E Test ==="

# Check binary exists
if [ ! -f "$BINARY" ]; then
    echo "Binary not found. Build first: cargo build --release"
    exit 1
fi

# Check tmux available
if ! command -v tmux &>/dev/null; then
    echo "tmux not found"
    exit 1
fi

CHECK_OUTPUT=$("$BINARY" --check 2>&1)
echo "$CHECK_OUTPUT"

# Verify output format
if echo "$CHECK_OUTPUT" | grep -q "tasks found"; then
    echo "E2E check: PASS"
else
    echo "E2E check: FAIL"
    exit 1
fi

# Test JSON mode
JSON_OUTPUT=$("$BINARY" --json 2>&1)
if echo "$JSON_OUTPUT" | grep -q "\["; then
    echo "JSON mode: PASS"
else
    echo "JSON mode: FAIL"
    exit 1
fi

# Test doctor
"$BINARY" --doctor 2>&1 || true

echo "=== E2E tests complete ==="
