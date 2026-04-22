#!/usr/bin/env bash
# DJS Bundler Test Suite
set +e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../.."
source "$HOME/.cargo/env" 2>/dev/null || true

PASS=0; FAIL=0; TOTAL=0
DJS="./target/release/djs"
BUNDLE_OUT="/tmp/djs-bundle-test"

bundler_test() {
    local name="$1" cmd="$2" check="$3"
    TOTAL=$((TOTAL + 1))
    
    local out exit_code=0
    out=$(eval "$cmd" 2>&1) || exit_code=$?
    
    if [ -z "$check" ]; then
        if [ "$exit_code" -eq 0 ]; then
            PASS=$((PASS + 1))
            echo "  ✓ $name"
        else
            FAIL=$((FAIL + 1))
            echo "  ✗ $name"
            echo "    exit code: $exit_code"
            echo "    output: $(echo "$out" | head -1)"
        fi
    elif echo "$out" | grep -q "$check"; then
        PASS=$((PASS + 1))
        echo "  ✓ $name"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $name"
        echo "    expected: $check"
        echo "    got:      $(echo "$out" | head -1)"
    fi
}

file_test() {
    local name="$1" file="$2" check="$3"
    TOTAL=$((TOTAL + 1))
    
    if [ -f "$file" ]; then
        local content
        content=$(cat "$file")
        if echo "$content" | grep -q "$check"; then
            PASS=$((PASS + 1))
            echo "  ✓ $name"
        else
            FAIL=$((FAIL + 1))
            echo "  ✗ $name — file missing: $check"
        fi
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $name — file not found: $file"
    fi
}

echo "Building djs..."
cargo build --release --quiet 2>/dev/null

echo ""
echo "=============================================="
echo " DJS Bundler Test Suite"
echo "=============================================="
echo ""

# Clean test directory
rm -rf "$BUNDLE_OUT"
mkdir -p "$BUNDLE_OUT"

# ── Basic Bundling ──
echo "── Basic Bundling ──"
bundler_test "djs build (simple entry)" \
    "$DJS build tests/bundler/test_entry.js $BUNDLE_OUT/bundle.js" \
    "Build successful"

file_test "bundle file created" \
    "$BUNDLE_OUT/bundle.js" \
    "DJS Production Bundle"

file_test "bundle contains entry code" \
    "$BUNDLE_OUT/bundle.js" \
    "function test()"

file_test "bundle contains imported module" \
    "$BUNDLE_OUT/bundle.js" \
    "function add(a, b)"

# ── Bundled File Execution ──
echo "── Bundled File Execution ──"
bundled_output=$($DJS run "$BUNDLE_OUT/bundle.js" 2>&1)
TOTAL=$((TOTAL + 1))
if echo "$bundled_output" | grep -q "90"; then
    PASS=$((PASS + 1))
    echo "  ✓ bundled file executes correctly"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ bundled file incorrect output"
    echo "    expected: 90"
    echo "    got:      $bundled_output"
fi

# ── Complex Module Tree ──
echo "── Complex Module Tree ──"
mkdir -p "$BUNDLE_OUT/modules"

# Create math module
cat > "$BUNDLE_OUT/modules/math.js" << 'EOF'
export function add(a, b) { return a + b; }
export function sub(a, b) { return a - b; }
export function mul(a, b) { return a * b; }
EOF

# Create fib module
cat > "$BUNDLE_OUT/modules/fib.js" << 'EOF'
export function fib(n) {
    return n > 1 ? fib(n - 1) + fib(n - 2) : n;
}
EOF

# Create entry that imports both
cat > "$BUNDLE_OUT/complex_entry.js" << 'EOF'
import { add, mul } from "./modules/math";
import { fib } from "./modules/fib";

function test(addFn, mulFn, fibFn) {
    return mulFn(addFn(10, 20), fibFn(5));
}
test(add, mul, fib)
EOF

bundler_test "djs build (complex tree)" \
    "$DJS build $BUNDLE_OUT/complex_entry.js $BUNDLE_OUT/complex_bundle.js" \
    "Build successful"

bundled_output=$($DJS run "$BUNDLE_OUT/complex_bundle.js" 2>&1)
TOTAL=$((TOTAL + 1))
if echo "$bundled_output" | grep -q "150"; then
    PASS=$((PASS + 1))
    echo "  ✓ complex bundle executes correctly (mul(add(10,20), fib(5)) = 150)"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ complex bundle incorrect output"
    echo "    expected: 150"
    echo "    got:      $bundled_output"
fi

# ── Circular Import Protection ──
echo "── Circular Import Protection ──"
mkdir -p "$BUNDLE_OUT/circular"

cat > "$BUNDLE_OUT/circular/a.js" << 'EOF'
import { funcB } from "./b";
export function funcA() { return 1; }
EOF

cat > "$BUNDLE_OUT/circular/b.js" << 'EOF'
import { funcA } from "./a";
export function funcB() { return 2; }
EOF

cat > "$BUNDLE_OUT/circular/entry.js" << 'EOF'
import { funcA } from "./a";
function test(f) { return f(); }
test(funcA)
EOF

bundler_test "djs build (circular imports)" \
    "$DJS build $BUNDLE_OUT/circular/entry.js $BUNDLE_OUT/circular_bundle.js" \
    "Build successful"

bundled_output=$($DJS run "$BUNDLE_OUT/circular_bundle.js" 2>&1)
TOTAL=$((TOTAL + 1))
if echo "$bundled_output" | grep -q "1"; then
    PASS=$((PASS + 1))
    echo "  ✓ circular imports handled correctly"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ circular imports failed"
    echo "    got: $bundled_output"
fi

# ── Summary ──
rm -rf "$BUNDLE_OUT"

echo ""
echo "=============================================="
echo " Bundler Results: $PASS passed, $FAIL failed, $TOTAL total"
echo "=============================================="

if [ "$FAIL" -gt 0 ]; then exit 1; fi
exit 0
