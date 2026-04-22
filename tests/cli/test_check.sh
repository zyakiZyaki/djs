#!/usr/bin/env bash
# DJS CLI Test Suite
set +e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../.."
source "$HOME/.cargo/env" 2>/dev/null || true

PASS=0; FAIL=0; TOTAL=0

# ── Helpers ──

cli_test() {
    local name="$1" cmd="$2" expected="$3" expect_fail="${4:-false}"
    TOTAL=$((TOTAL + 1))
    
    local out exit_code=0
    out=$(eval "$cmd" 2>&1) || exit_code=$?
    
    if [ "$expect_fail" = "true" ]; then
        if [ "$exit_code" -ne 0 ]; then
            PASS=$((PASS + 1))
            echo "  ✓ $name"
        else
            FAIL=$((FAIL + 1))
            echo "  ✗ $name — expected failure but succeeded"
        fi
    else
        if echo "$out" | grep -q "$expected"; then
            PASS=$((PASS + 1))
            echo "  ✓ $name"
        else
            FAIL=$((FAIL + 1))
            echo "  ✗ $name"
            echo "    expected: $expected"
            echo "    got:      $(echo "$out" | head -1)"
        fi
    fi
}

# ── Build ──
echo "Building djs..."
cargo build --release --quiet 2>/dev/null
DJS="./target/release/djs"

if [ ! -f "$DJS" ]; then
    echo "ERROR: djs binary not found!"
    exit 1
fi

echo ""
echo "=============================================="
echo " DJS CLI Test Suite"
echo "=============================================="
echo ""

# ── Version ──
echo "── Version ──"
cli_test "djs version" "$DJS version" "DJS v0.1.0"
cli_test "djs -v" "$DJS -v" "DJS v0.1.0"
cli_test "djs --version" "$DJS --version" "DJS v0.1.0"

# ── Help ──
echo "── Help ──"
cli_test "djs help" "$DJS help" "DJS - Declarative JavaScript VM"
cli_test "djs -h" "$DJS -h" "djs run"
cli_test "djs --help" "$DJS --help" "djs build"

# ── Run ──
echo "── Run ──"
cli_test "djs run (basic)" "$DJS run tests/language/unit/arithmetic.js" "5"
cli_test "djs run (fib)" "$DJS run tests/language/unit/recursion_fib.js" "55"
cli_test "djs run (closure)" "$DJS run tests/language/unit/closure_capture.js" "150"
cli_test "djs run (module)" "$DJS run tests/language/modules/test_import_basic.js" "90"

# ── Backward Compatibility ──
echo "── Backward Compatibility ──"
cli_test "djs <file> (no run)" "$DJS tests/language/unit/arithmetic.js" "5"
cli_test "djs --vm <file>" "$DJS --vm tests/language/unit/arithmetic.js" "5"

# ── Build ──
echo "── Build ──"
rm -rf /tmp/djs-build-test
cli_test "djs build (creates output)" \
    "$DJS build tests/language/modules/test_import_basic.js /tmp/djs-build-test/out.js" \
    "Build successful"
    
if [ -f "/tmp/djs-build-test/out.js" ]; then
    TOTAL=$((TOTAL + 1))
    if grep -q "DJS Production Bundle" /tmp/djs-build-test/out.js; then
        PASS=$((PASS + 1))
        echo "  ✓ build output contains bundle header"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ build output missing bundle header"
    fi
else
    TOTAL=$((TOTAL + 1))
    FAIL=$((FAIL + 1))
    echo "  ✗ build output file not created"
fi

cli_test "djs build (default output)" \
    "$DJS build tests/language/unit/arithmetic.js" \
    "dist/bundle.js"

# ── Build Errors ──
echo "── Build Errors ──"
cli_test "djs build (missing file)" \
    "$DJS build nonexistent.js" \
    "No such file" \
    "true"

cli_test "djs build (no args)" \
    "$DJS build" \
    "requires an entry file" \
    "true"

# ── Run Errors ──
echo "── Run Errors ──"
cli_test "djs run (missing file)" \
    "$DJS run nonexistent.js" \
    "No such file" \
    "true"

cli_test "djs run (no args)" \
    "$DJS run" \
    "requires a file argument" \
    "true"

# ── Check ──
echo "── Check ──"
cli_test "djs check (single file)" \
    "$DJS check tests/language/unit/arithmetic.js" \
    "Checking:"

cli_test "djs check (directory)" \
    "$DJS check tests/language/unit/" \
    "Checking files in:"

# ── REPL (smoke test) ──
echo "── REPL ──"
cli_test "djs repl starts" \
    "echo '1+1' | $DJS repl" \
    "DJS REPL"

# ── Deploy ──
echo "── Deploy ──"
TOTAL=$((TOTAL + 1))
DJS_ABS="$SCRIPT_DIR/../../target/release/djs"
out=$(rm -rf /tmp/djs-deploy-test && mkdir -p /tmp/djs-deploy-test && cp tests/language/unit/arithmetic.js /tmp/djs-deploy-test/ && cd /tmp/djs-deploy-test && $DJS_ABS deploy arithmetic.js 2>&1)
if echo "$out" | grep -q "Deployed successfully"; then
    PASS=$((PASS + 1))
    echo "  ✓ djs deploy succeeds"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ djs deploy failed"
    echo "    got:      $(echo "$out" | head -1)"
fi

# ── Summary ──
echo ""
echo "=============================================="
echo " Check Results: $PASS passed, $FAIL failed, $TOTAL total"
echo "=============================================="

if [ "$FAIL" -gt 0 ]; then exit 1; fi
exit 0
