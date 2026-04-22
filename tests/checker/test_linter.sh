#!/usr/bin/env bash
# DJS Linter Test Suite
set +e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../.."
source "$HOME/.cargo/env" 2>/dev/null || true

PASS=0; FAIL=0; TOTAL=0
DJS="./target/release/djs"
LINT_OUT="/tmp/djs-lint-test"

lint_test() {
    local name="$1" cmd="$2" expect_fail="${3:-false}"
    TOTAL=$((TOTAL + 1))
    
    local out exit_code=0
    out=$(eval "$cmd" 2>&1) || exit_code=$?
    
    if [ "$expect_fail" = "true" ]; then
        if [ "$exit_code" -ne 0 ] && echo "$out" | grep -q "error(s)"; then
            PASS=$((PASS + 1))
            echo "  ✓ $name"
        else
            FAIL=$((FAIL + 1))
            echo "  ✗ $name — expected lint errors"
            echo "    got:      $(echo "$out" | head -1)"
        fi
    else
        if [ "$exit_code" -eq 0 ]; then
            PASS=$((PASS + 1))
            echo "  ✓ $name"
        else
            FAIL=$((FAIL + 1))
            echo "  ✗ $name"
            echo "    got:      $(echo "$out" | head -1)"
        fi
    fi
}

echo "Building djs..."
cargo build --release --quiet 2>/dev/null

echo ""
echo "=============================================="
echo " DJS Linter Test Suite"
echo "=============================================="
echo ""

rm -rf "$LINT_OUT"
mkdir -p "$LINT_OUT"

# ── Clean Code (should pass lint) ──
echo "── Clean Code ──"
cat > "$LINT_OUT/clean.js" << 'EOF'
import { add } from "./math";
function test(fn) {
    return fn(1, 2);
}
test(add)
EOF

cat > "$LINT_OUT/math.js" << 'EOF'
export function add(a, b) { return a + b; }
EOF

lint_test "clean code passes lint" \
    "$DJS lint $LINT_OUT/clean.js"

# ── Impure Function (should fail lint) ──
echo "── Impure Functions ──"
cat > "$LINT_OUT/dirty.js" << 'EOF'
import { add } from "./math";
function test() {
    return add(1, 2);
}
test()
EOF

lint_test "impure function fails lint" \
    "$DJS lint $LINT_OUT/dirty.js" \
    "true"

# ── Mixed Clean and Dirty ──
echo "── Mixed Code ──"
cat > "$LINT_OUT/mixed.js" << 'EOF'
import { add, mul } from "./math";
function clean(fn) {
    return fn(1, 2);
}
function dirty() {
    return add(3, 4);
}
clean(mul)
EOF

lint_test "mixed code fails lint" \
    "$DJS lint $LINT_OUT/mixed.js" \
    "true"

# ── Multiple Imports ──
echo "── Multiple Imports ──"
cat > "$LINT_OUT/multi_import.js" << 'EOF'
import { add, mul, sub } from "./math";
function test(a, b, c) {
    return a(b(1, 2), c(3, 4));
}
test(add, mul, sub)
EOF

lint_test "multiple imports clean" \
    "$DJS lint $LINT_OUT/multi_import.js"

# ── No Imports (trivially clean) ──
echo "── No Imports ──"
cat > "$LINT_OUT/no_imports.js" << 'EOF'
function test() {
    return 42;
}
test()
EOF

lint_test "no imports passes lint" \
    "$DJS lint $LINT_OUT/no_imports.js"

# ── Directory Lint ──
echo "── Directory Lint ──"
mkdir -p "$LINT_OUT/subdir"
cat > "$LINT_OUT/subdir/good.js" << 'EOF'
function test() { return 1; }
test()
EOF

lint_test "directory lint (clean)" \
    "$DJS lint $LINT_OUT/subdir"

# ── Summary ──
rm -rf "$LINT_OUT"

echo ""
echo "=============================================="
echo " Linter Results: $PASS passed, $FAIL failed, $TOTAL total"
echo "=============================================="

if [ "$FAIL" -gt 0 ]; then exit 1; fi
exit 0
