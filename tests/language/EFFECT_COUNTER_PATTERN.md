# Counter Effect Pattern - Test Suite

## Overview
This test suite demonstrates the counter effect pattern in DJS, where callbacks are used to handle state changes in a pure functional way.

## Pattern Description
```javascript
function counter(eff) {
    return function step(count) {
        return {
            inc: function () {
                eff(count + 1);  // Effect: notify of state change
                return step(count + 1);  // Return new state accessor
            },
            dec: function () {
                eff(count - 1);
                return step(count - 1);
            }
        };
    };
}
```

The `eff` callback is invoked whenever the state changes, allowing side effects (logging, accumulation, etc.) while maintaining pure functional semantics.

## Test Files

### Unit Tests
1. `effect_counter_basic.js` - Basic inc/dec chain (requires VM fix for nested closure methods)
2. `effect_counter_mixed.js` - Mixed inc/dec operations
3. `effect_counter_accumulate.js` - Effect accumulation pattern
4. `effect_counter_object.js` - Effect returning state objects
5. `effect_simple.js` - Simple effect call pattern
6. `effect_counter_working.js` - Working version using named functions

### Integration Tests  
1. `effect_counter_console.js` - Counter with console.log tracing
2. `effect_counter_reduce.js` - Counter operations via function composition

### Negative Tests
1. `effect_counter_non_func.js` - Passing non-function as effect (should fail)

## Language Features Added

### 1. Expression Statements (`Stmt::Expr`)
- Allows function bodies to have expressions that don't return
- Smart parsing: last expression becomes return, others are expression statements
- Enables the `eff(x); return y;` pattern

### 2. Closure Method Calls
- Objects can now have closure properties that are callable as methods
- Added support in `exec_method` for calling `BytecodeClosure` properties
- Proper stack management for method execution

### 3. Enhanced Capture Analysis
- Extended capture analysis to handle `Stmt::Expr` in addition to `Stmt::Return`
- Recursive capture for nested function declarations

## Current Limitations

### Nested Closure Method Issue
The pattern `step` (named function) returning an object with a method that calls a captured function has a VM limitation. The captured function isn't being resolved correctly in this specific nested scenario.

**Workaround**: Use named function declarations instead of inline anonymous functions:
```javascript
function step(count) {
    function doInc() {
        eff(count + 1);
        return step(count + 1);
    }
    return {inc: doInc};
}
```

However, anonymous functions can't reference themselves by name for recursion.

## Implementation Files Modified

1. `src/parser.rs` - Added `Stmt::Expr`, smart expression statement handling
2. `src/vm/compiler.rs` - Handle `Stmt::Expr` in compilation and capture analysis
3. `src/vm/vm.rs` - Added closure method call support in `exec_method`
4. `tests/language/unit/` - Added counter effect pattern tests
5. `tests/language/integration/` - Added integration tests
6. `tests/language/negative/` - Added negative test for non-function effects

## Running Tests

```bash
# Run all tests
bash tests/language/test.sh

# Run specific test
./target/release/djs --vm tests/language/unit/effect_simple.js
```

## Future Work

To fully support the counter effect pattern as originally designed:
1. Fix nested closure capture resolution in the VM
2. Add support for named function expressions (self-referential anonymous functions)
3. Consider adding a proper effect system with `perform`/`handle` syntax
