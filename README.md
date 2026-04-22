# DJS (Declarative JavaScript) VM

> Bytecode virtual machine for Declarative JavaScript, written in Rust.
> Pure functions, immutable data, module system, fetch API, built-in test framework.

## Overview

DJS is a bytecode virtual machine that executes a declarative subset of JavaScript with the following key features:

- Pure functions only (no `let`, `var`, mutations)
- Immutable data structures
- First-class functions with closures and currying
- Module system via source concatenation
- Built-in fetch API for HTTP requests
- HTTP server with handler closure requirement
- Synchronous promises (no event loop)
- Built-in test framework with expect chaining

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/zyakiZyaki/djs.git
cd djs

# Build the binary
cargo build --release

# Install to system path (optional)
cp target/release/djs /usr/local/bin/djs
```

### Using install.sh

```bash
# Download and install with one command
sh -c "$(curl -fsSL https://raw.githubusercontent.com/zyakiZyaki/djs/master/install.sh)"
```

## Usage

### Basic Execution

```bash
djs file.js
# or
djs run file.js
```

### Deploy Pipeline

```bash
djs deploy
```

This runs the following steps:
1. Check: Verify file compiles and executes
2. Lint: Check for impure functions
3. Build: Bundle all modules into dist/bundle.js

### Build Project

```bash
djs build src/main.js
```

### Run Tests

```bash
djs test tests/
```

### Lint Project

```bash
djs lint src/
```

### Interactive REPL

```bash
djs repl
```

### Help

```bash
djs help
```

## Language Features

### Core
- Pure functions only — no `let`, `var`, `class`, mutations
- Immutable data — arrays and objects are never modified in place
- First-class functions — closures, currying, higher-order functions
- Destructuring — `function getName({name, age}) { return name; }`
- Rest params — `function sum(...args) { ... }`
- Spread operator — `[...arr1, ...arr2]`, `fn(...args)`
- Ternary expressions — `cond ? then : else` (no `if/else` statements)
- Expression statements — expressions without return (for effects)

### Module System
```javascript
// math.js
export function add(a, b) { return a + b; }
export function sub(a, b) { return a - b; }

// main.js
import { add, sub } from "./math";
function test(addFn, subFn) {
  return addFn(10, subFn(5, 3));
}
test(add, sub)
```

- Relative imports with automatic `.js` extension resolution
- Recursive dependency resolution with cycle detection
- **Purity rule**: imports must be passed as parameters, not captured by functions

### Fetch API
```javascript
// GET request
fetch("https://api.example.com/data").then(function(res) {
  return res.json();
});

// POST with JSON body
fetch("https://api.example.com/data", {
  method: "POST",
  body: { title: "test", userId: 1 }
}).then(function(res) {
  return res.status; // → 201
});
```

Fetch returns a **Promise** that resolves synchronously with the Response object.

### HTTP Server
```javascript
http
.createServer(function handler(req) {
  console.log("Request:", req.url);
  return {
    status: 200,
    headers: { "Content-Type": "text/plain" },
    body: "Hello, World!"
  };
})
.listen(8888);
```

The handler **must** be a closure defined inside another function (nesting_depth > 0).

### Promises
```javascript
new Promise(function(resolve, reject) {
  resolve(42);
}).then(function(x) {
  return x * 2;
});
// → 84
```

Promises are executed **synchronously** in this VM — no async/await or event loop.

### JSON
```javascript
JSON.parse('{"name": "max", "age": 30}')
JSON.stringify({name: "max", age: 30})
```

### Built-in Test Framework

No imports needed — `test`, `expect`, `describe` are globals:

```javascript
describe("math", function() {
  test("addition", function() {
    expect(1 + 2).toEqual(3);
  });
  test("not equal", function() {
    expect(1).not.toEqual(2);
  });
});
```

| Function | Description |
|----------|-------------|
| `test(name, fn)` | Run test, print ✓ or ✗ |
| `expect(value)` | Create expect object for chaining |
| `describe(name, fn)` | Group tests with a header |

#### Matchers

| Matcher | Description |
|---------|-------------|
| `expect(a).toEqual(b)` | Deep equality |
| `expect(a).toBe(b)` | Alias for toEqual |
| `expect(a).not.toEqual(b)` | Negation |
| `expect(a).toBeTruthy()` | Truthy check |
| `expect(a).toBeFalsy()` | Falsy check |
| `expect(a).toBeGreaterThan(b)` | Number comparison |
| `expect(a).toBeLessThan(b)` | Number comparison |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Source Code (.js) │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ Lexer (lexer.rs) │
│ ───────────────── │
│ Input: raw text │
│ Output: Vec<Token> │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ Parser (parser.rs) │
│ ────────── │
│ Input: Vec<Token> │
│ Output: FuncDecl + Expr (AST) │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ Compiler (vm/compiler.rs) │
│ ──────────────── │
│ Input: AST (FuncDecl + Expr) │
│ Output: Chunk (bytecode) + Vec<CompiledFunction> │
│ │
│ Scope management with parent chain for closures. │
│ Captured variables tracked at compile time. │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ VM (vm/vm.rs) │
│ ── │
│ Input: Chunk + Vec<CompiledFunction> │
│ Output: Value │
│ │
│ Stack-based architecture: │
│ - value_stack: Vec<Value> (operand stack) │
│ - frames: Vec<CallFrame> (call stack) │
│ - globals: HashMap<String, Value> │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│ Result (printed) │
└─────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Flat locals per frame (not linked environments)
Each `CallFrame` has a flat `Vec<Option<Value>>` for locals, not a chain of parent environments. This makes variable lookup O(1) instead of O(depth).

### 2. Captured values by slot index, not by name
The compiler tracks `(parent_slot, new_slot)` pairs for each captured variable. At runtime, `MakeClosure` copies values directly from the parent frame's locals.

### 3. Promises executed synchronously
In this VM, `new Promise(executor)` calls the executor immediately and `.then(callback)` invokes the callback synchronously if the promise is already fulfilled.

### 4. Single value stack + call frames
The VM uses one shared `value_stack: Vec<Value>` for operand storage, separate from the `frames: Vec<CallFrame>` call stack.

### 5. No garbage collector (Rust ownership)
Values are `Clone` types (numbers, strings, HashMaps via `Rc` where needed). When a frame is popped, its locals are dropped. No tracing GC needed.

### 6. Module system via source concatenation
Instead of complex bytecode linking, the module system concatenates source files (with imports stripped) and compiles as one program.

## Release Information

- **Version**: v0.1.0
- **Binary**: `target/release/djs`
- **Installation**: `install.sh`
- **Repository**: https://github.com/zyakiZyaki/djs
- **Author**: MaxShe

## License

MIT
