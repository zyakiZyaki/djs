#!/usr/bin/env bash
# Run HTTP tests against local server
set +e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVER_PID=""
cleanup() {
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null
    rm -f /tmp/server.log
}
trap cleanup EXIT

cd "$(dirname "$0")/.."
PROJECT_DIR="$(pwd)"
cargo build --release --quiet 2>/dev/null

echo "Starting test server on port 8888..."
node "$SCRIPT_DIR/server.js" > /tmp/server.log 2>&1 &
SERVER_PID=$!
sleep 1

if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "Failed to start server!"
    cat /tmp/server.log
    exit 1
fi

echo "Server running (PID $SERVER_PID)"
echo ""

PASS=0
FAIL=0
TOTAL=0

run_test() {
    local name="$1" expected="$2"
    TOTAL=$((TOTAL + 1))
    local out
    out=$("$PROJECT_DIR/target/release/djs" --vm "$PROJECT_DIR/tests/language/fetch/$name" 2>&1) || true

    if echo "$out" | grep -q "$expected"; then
        PASS=$((PASS + 1))
        echo "  ✓ $name  → $expected"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $name"
        echo "    expected: $expected"
        echo "    got:      $(echo "$out" | head -1)"
    fi
}

echo "Running HTTP tests against localhost:8888"
echo ""

echo "── HTTP GET ──"
run_test "test_http_hello.js" "Hello, World!"
run_test "test_http_health.js" "object"

echo "── HTTP POST ──"
run_test "test_http_echo.js" "object"

echo "── HTTP PUT ──"
run_test "test_http_put.js" "object"

echo "── HTTP DELETE ──"
run_test "test_http_delete.js" "204"

echo ""
echo "========================================"
echo " HTTP Tests: $PASS passed, $FAIL failed, $TOTAL total"
echo "========================================"

if [ "$FAIL" -gt 0 ]; then exit 1; fi
exit 0
