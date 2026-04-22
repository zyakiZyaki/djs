#!/usr/bin/env bash
# DJS — VM Language Test Suite
set +e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../.."
source "$HOME/.cargo/env" 2>/dev/null || true

PASS=0; FAIL=0; TOTAL=0

# ── Helpers ──

test_pass() {
    local file="$1" expected="$2"
    TOTAL=$((TOTAL + 1))
    local out
    out=$(./target/release/djs --vm "$file" 2>&1) || true
    if [ "$out" = "$expected" ]; then
        PASS=$((PASS + 1))
        echo "  ✓ $(basename "$file")"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $(basename "$file")"
        echo "    expected: $expected"
        echo "    got:      $(echo "$out" | head -1)"
    fi
}

test_fail() {
    local file="$1"
    TOTAL=$((TOTAL + 1))
    local exit_code=0
    ./target/release/djs --vm "$file" >/dev/null 2>&1 || exit_code=$?
    if [ "$exit_code" -ne 0 ]; then
        PASS=$((PASS + 1))
        echo "  ✓ $(basename "$file")"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ $(basename "$file") — should have failed"
    fi
}

# ── Build ──
echo "Building djs..."
cargo build --release --quiet 2>/dev/null

echo ""
echo "=============================================="
echo " DJS — VM Test Suite"
echo "=============================================="
echo ""

echo "── Unit: Operators ──"
test_pass "tests/language/unit/arithmetic.js" "5"
test_pass "tests/language/unit/subtraction.js" "6"
test_pass "tests/language/unit/multiplication.js" "21"
test_pass "tests/language/unit/division.js" "5"
test_pass "tests/language/unit/modulo.js" "1"
test_pass "tests/language/unit/comparison_eq.js" "true"
test_pass "tests/language/unit/comparison_ne.js" "true"
test_pass "tests/language/unit/comparison_lt.js" "true"
test_pass "tests/language/unit/comparison_gt.js" "true"
test_pass "tests/language/unit/bool_and.js" "true"
test_pass "tests/language/unit/bool_or.js" "true"
test_pass "tests/language/unit/ternary_true.js" "100"
test_pass "tests/language/unit/ternary_false.js" "200"
test_pass "tests/language/unit/ternary_cond.js" "42"
test_pass "tests/language/unit/number_edge.js" "[-2, 4, 10, 25]"
test_pass "tests/language/unit/string_comparison.js" "true"
test_pass "tests/language/unit/bool_edge.js" "[true, false, true, false, true, false]"
test_pass "tests/language/unit/string_number_mix.js" "count: 42 items"
test_pass "tests/language/unit/ternary_in_call.js" "42"
test_pass "tests/language/unit/index_negative.js" "1"
test_pass "tests/language/unit/conditional_nested.js" "[-1, 0, 1, 2, 3]"

echo "── Unit: Strings ──"
test_pass "tests/language/unit/strings_double.js" "hello"
test_pass "tests/language/unit/strings_single.js" "world"
test_pass "tests/language/unit/strings_concat.js" "hello world"
test_pass "tests/language/unit/strings_index.js" "e"
test_pass "tests/language/unit/strings_length.js" "5"
test_pass "tests/language/unit/strings_split.js" '[a, b, c]'

echo "── Unit: Arrays ──"
test_pass "tests/language/unit/array_literal.js" "[1, 2, 3]"
test_pass "tests/language/unit/array_empty.js" "[]"
test_pass "tests/language/unit/array_index.js" "20"
test_pass "tests/language/unit/array_length.js" "3"
test_pass "tests/language/unit/array_push.js" "[1, 2, 3]"
test_pass "tests/language/unit/array_concat.js" "[1, 2, 3]"
test_pass "tests/language/unit/array_join.js" "1-2-3"
test_pass "tests/language/unit/array_map.js" "[2, 4, 6]"
test_pass "tests/language/unit/array_filter.js" "[3, 4, 5]"
test_pass "tests/language/unit/array_reduce.js" "15"
test_pass "tests/language/unit/array_chain.js" "[30, 40, 50]"
test_pass "tests/language/unit/array_nested.js" "2"
test_pass "tests/language/unit/array_empty_ops.js" "[0, [1], 0, ]"
test_pass "tests/language/unit/array_reduce_complex.js" "[object]"
test_pass "tests/language/unit/map_with_destructure.js" "[10, 20, 30]"

echo "── Unit: Objects ──"
test_pass "tests/language/unit/object_prop.js" "max"
test_pass "tests/language/unit/object_bracket.js" "30"
test_pass "tests/language/unit/object_shorthand.js" "[object]"
test_pass "tests/language/unit/object_keys.js" "2"
test_pass "tests/language/unit/object_values.js" "2"
test_pass "tests/language/unit/object_nested.js" "max"
test_pass "tests/language/unit/object_destructure.js" "bob"
test_pass "tests/language/unit/object_multi_destructure.js" "35"
test_pass "tests/language/unit/object_empty.js" "2"
test_pass "tests/language/unit/object_chained.js" "42"
test_pass "tests/language/unit/object_nested_destructure.js" "[object]"

echo "── Unit: Spread & Rest ──"
test_pass "tests/language/unit/spread_array.js" "[1, 2, 3, 4, 5]"
test_pass "tests/language/unit/spread_call.js" "60"
test_pass "tests/language/unit/rest_single.js" "[2, 3, 4]"
test_pass "tests/language/unit/rest_reduce.js" "[1, 2, 3, 4, 5, 6]"
test_pass "tests/language/unit/spread_empty.js" "[1, 2]"
test_pass "tests/language/unit/rest_none.js" "[]"
test_pass "tests/language/unit/rest_destructure.js" "60"

echo "── Unit: Functions ──"
test_pass "tests/language/unit/recursion_fib.js" "55"
test_pass "tests/language/unit/recursion_fact.js" "720"
test_pass "tests/language/unit/curry.js" "15"
test_pass "tests/language/unit/curry_chain.js" "60"
test_pass "tests/language/unit/function_as_param.js" "42"
test_pass "tests/language/unit/recursion_deep.js" "5050"

echo "── Unit: Closures ──"
test_pass "tests/language/unit/closure_capture.js" "150"
test_pass "tests/language/unit/closure_nested.js" "11"
test_pass "tests/language/unit/closure_callback.js" "12"
test_pass "tests/language/unit/closure_deep.js" "6"
test_pass "tests/language/unit/closure_in_array.js" "3"
test_pass "tests/language/unit/closure_with_array.js" "[7, 10, 8]"
test_pass "tests/language/unit/filter_with_closure.js" "[10, 8]"
test_pass "tests/language/unit/closure_call_captured.js" "99"

echo "── Unit: Effect Patterns ──"
test_pass "tests/language/unit/effect_simple.js" "100"
# effect_counter_full.js prints to stderr, so we capture only stdout
TOTAL=$((TOTAL + 1))
out=$(./target/release/djs --vm tests/language/unit/effect_counter_full.js 2>/dev/null)
if [ "$out" = "[object]" ]; then
    PASS=$((PASS + 1))
    echo "  ✓ effect_counter_full.js"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ effect_counter_full.js"
    echo "    expected: [object]"
    echo "    got:      $(echo "$out" | tail -1)"
fi
test_pass "tests/language/unit/method_simple.js" "42"
# method_with_log.js prints to stderr, so we capture only stdout
TOTAL=$((TOTAL + 1))
out=$(./target/release/djs --vm tests/language/unit/method_with_log.js 2>/dev/null)
if [ "$out" = "42" ]; then
    PASS=$((PASS + 1))
    echo "  ✓ method_with_log.js"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ method_with_log.js"
    echo "    expected: 42"
    echo "    got:      $(echo "$out" | head -1)"
fi
test_pass "tests/language/unit/method_call_captured.js" "20"
test_pass "tests/language/unit/captured_call_in_nested.js" "1"
test_pass "tests/language/unit/captured_call_in_method.js" "1"
test_pass "tests/language/unit/captured_call_zeroparam.js" "99"
test_pass "tests/language/unit/counter_no_effect.js" "[object]"
test_pass "tests/language/unit/object_closure_method.js" "11"

echo "── Unit: Promises ──"
test_pass "tests/language/unit/promises.js" "42"
test_pass "tests/language/unit/promises_chain.js" "25"
test_pass "tests/language/unit/promises_catch.js" "oops"
test_pass "tests/language/unit/promises_finally.js" "1"
test_pass "tests/language/unit/promises_nested.js" "2"

echo "── Unit: JSON ──"
test_pass "tests/language/unit/json_parse_access.js" "max"
test_pass "tests/language/unit/json_stringify.js" "32"
test_pass "tests/language/unit/json_roundtrip.js" "[object]"

echo "── Negative Tests ──"
test_fail "tests/language/negative/global_var.js"
test_fail "tests/language/negative/global_call.js"
test_fail "tests/language/negative/undefined_param.js"
test_fail "tests/language/negative/parent_call.js"
test_fail "tests/language/negative/undefined_in_body.js"
test_fail "tests/language/negative/nested_outer_call.js"
test_fail "tests/language/negative/sibling_call.js"
test_fail "tests/language/negative/global_var_access.js"
test_fail "tests/language/negative/http_global_handler.js"

echo "── Integration Tests ──"
test_pass "tests/language/integration/closure_reduce.js" "60"
test_pass "tests/language/integration/spread_curry.js" "60"
test_pass "tests/language/integration/nested_closures.js" "6"
test_pass "tests/language/integration/json_objects.js" "max"
test_pass "tests/language/integration/json_arrays.js" "[2, 4, 6, 8, 10]"
test_pass "tests/language/integration/array_chain.js" "[9, 16, 25]"
test_pass "tests/language/integration/rest_destructure.js" "6"
test_pass "tests/language/integration/callback_chain.js" "1110"
test_pass "tests/language/integration/closure_rest.js" "[1, 2, 3, 4]"
test_pass "tests/language/integration/nested_rest.js" "60"
test_pass "tests/language/integration/callback_pattern.js" "done"
test_pass "tests/language/integration/closure_chain.js" "20"
test_pass "tests/language/integration/json_complex.js" "[object]"

echo "── Module Tests ──"
test_pass "tests/language/modules/test_import_basic.js" "90"
test_pass "tests/language/modules/test_import_fib.js" "55"

echo "── Console Tests ──"
# console.log prints to stderr, stdout has the return value
TOTAL=$((TOTAL + 1))
out=$(./target/release/djs --vm tests/language/fetch/test_console_log.js 2>/dev/null)
if [ "$out" = "99" ]; then
    PASS=$((PASS + 1))
    echo "  ✓ test_console_log.js"
else
    FAIL=$((FAIL + 1))
    echo "  ✗ test_console_log.js"
    echo "    expected: 99"
    echo "    got:      $out"
fi

# Fetch tests require internet connection
# Uncomment to test against live API:
# echo "── Fetch Tests ──"
# test_pass "tests/language/fetch/test_fetch_basic.js" "[object]"
# test_pass "tests/language/fetch/test_fetch_post.js" "201"
# test_pass "tests/language/fetch/test_fetch_text.js" "[value]"

echo ""
echo "=============================================="
echo " VM Results: $PASS passed, $FAIL failed, $TOTAL total"
echo "=============================================="

if [ "$FAIL" -gt 0 ]; then exit 1; fi
exit 0
