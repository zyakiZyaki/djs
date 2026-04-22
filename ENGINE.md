# DJS (Declarative JavaScript) VM Engine

> Bytecode virtual machine for Declarative JavaScript.
> Pure functions, immutable data, module system, fetch API, built-in test framework.

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

## Project Structure

```
src/
├── lib.rs # Library root — pub mod declarations
├── main.rs # CLI entry point (~680 lines)
├── lexer.rs # Token enum + Lexer
├── parser.rs # AST types + Parser
├── values.rs # Runtime values + environment
│ # - Value enum (12 variants including Expect)
│ # - Closure, PromiseState
│ # - Env (HashMap-based, with parent chain)
└── vm/
    ├── mod.rs # VM module root
    ├── opcode.rs # Bytecode instruction set (25 opcodes)
    ├── compiler.rs # AST → Bytecode compiler
    ├── vm.rs # Bytecode virtual machine (~2200 lines)
    ├── builtins.rs # Native functions (fetch, HTTP server)
    ├── module.rs # Module system (import/export)
    └── tests.rs # VM unit tests

tests/
├── language/
│   ├── test.sh # Language test runner (117 tests)
│   ├── unit/ # Unit tests
│   ├── integration/ # Integration tests
│   ├── negative/ # Error cases
│   ├── modules/ # Import/export tests
│   └── fetch/ # Fetch API tests
├── cli/
│   └── test_check.sh # CLI command tests (23 tests)
├── bundler/
│   └── test_bundler.sh # Bundler tests (9 tests)
├── checker/
│   └── test_linter.sh # Linter tests (6 tests)
└── test_framework/ # Test framework tests (9 tests)
    ├── test_expect_equal.js
    ├── test_expect_not.js
    └── test_expect_comparison.js
```

---

## Language Features

### Core
- **Pure functions only** — no `let`, `var`, `class`, mutations
- **Immutable data** — arrays and objects are never modified in place
- **First-class functions** — closures, currying, higher-order functions
- **Destructuring** — `function getName({name, age}) { return name; }`
- **Rest params** — `function sum(...args) { ... }`
- **Spread operator** — `[...arr1, ...arr2]`, `fn(...args)`
- **Ternary expressions** — `cond ? then : else` (no `if/else` statements)
- **Expression statements** — expressions without return (for effects)

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
- **Purity rule:** imports must be passed as parameters, not captured by functions

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

---

## Bytecode Instruction Set (25 Opcodes)

### Stack Operations
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `PushConst(n)` | `→ [const[n]]` | Push constant from pool |
| `Dup` | `[a] → [a, a]` | Duplicate top of stack |
| `Pop` | `[a] → []` | Discard top of stack |

### Variables
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `LoadLocal(slot)` | `→ [value]` | Load from frame's local array |
| `StoreLocal(slot)` | `[v] → [v]` | Store into frame's local array |
| `LoadGlobal(idx)` | `→ [value]` | Load from global environment |
| `StoreGlobal(idx)` | `[v] → [v]` | Store into global environment |

### Arithmetic & Comparison
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `Add` | `[a, b] → [a+b]` | Add or concatenate |
| `Sub` | `[a, b] → [a-b]` | Subtract |
| `Mul` | `[a, b] → [a*b]` | Multiply |
| `Div` | `[a, b] → [a/b]` | Divide |
| `Mod` | `[a, b] → [a%b]` | Modulo |
| `Eq` | `[a, b] → [bool]` | Equal |
| `Ne` | `[a, b] → [bool]` | Not equal |
| `Lt` | `[a, b] → [bool]` | Less than |
| `Gt` | `[a, b] → [bool]` | Greater than |
| `Le` | `[a, b] → [bool]` | Less or equal |
| `Ge` | `[a, b] → [bool]` | Greater or equal |

### Control Flow
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `Jump(t)` | — | Unconditional jump to offset `t` |
| `JumpIfFalse(t)` | `[cond] → [cond]` | Jump if top is falsey (keeps value) |
| `JumpIfTrue(t)` | `[cond] → [cond]` | Jump if top is truthy (keeps value) |
| `JumpIfFalsePop(t)` | `[cond] →` | Jump if falsey, pops value |
| `JumpIfTruePop(t)` | `[cond] →` | Jump if truthy, pops value |
| `ToBool` | `[v] → [bool]` | Convert to boolean |

### Data Structures
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `PushDepthMarker` | `→ [marker]` | Push sentinel for array construction |
| `MakeArray` | `[m, e1, e2, ...] → [arr]` | Collect elements until marker |
| `MakeObject(n)` | `[k1, v1, ..., kn, vn] → [obj]` | Build object from n key-value pairs |
| `Index` | `[base, idx] → [value]` | Array/string/object indexing |
| `GetProperty(i)` | `[obj] → [value]` | Get property by name from pool |
| `Spread` | `[arr] → [e1, e2, ...]` | Expand array into individual values |

### Functions
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `MakeClosure(fn)` | `→ [closure]` | Create closure for function index |
| `Call(n)` | `[fn, a1, ..., an] → [result]` | Call function with n args |
| `CallSpread(n)` | `[fn, a1, ...elems] → [result]` | Call with spread-expanded args |
| `TailCall(n)` | `[fn, a1, ..., an] → [result]` | Tail-optimized call (reuses frame) |
| `TailCallSpread(n)` | `[fn, ...] → [result]` | Tail call with spread |
| `MethodCall(n)` | `[obj, method, a1, ...] → [result]` | Call method on object |
| `Return` | `[value] →` (frame pops) | Return from function |

### Promises
| Opcode | Stack Effect | Description |
|--------|-------------|-------------|
| `MakePromise` | `[executor] → [promise]` | Create Promise, call executor(resolve, reject) |

---

## VM Execution Model

### Call Frame Layout

Each function call creates a `CallFrame`:

```
CallFrame {
  fn_idx: 3, // Index into functions[]
  ip: 14, // Current instruction offset
  locals: [ // Flat array, not HashMap
    0: Some(BytecodeClosure { fn_idx: 3, captured: [...] }), // self
    1: Some(Number(10.0)), // param "n"
    2: None, // (unused)
    3: Some(Number(42.0)), // captured variable "x"
    ...
  ]
}
```

### Function Call Sequence

```
1. Compiler generates: LoadLocal(0) PushConst(0) Call(1)
Stack: [closure, 10]

2. VM executes Call(1):
func_pos = stack.len() - 1 - 1 = 0
func_val = stack[0] // closure
args = drain(1..) = [Number(10)]

3. New frame created:
locals[0] = Some(closure) // self for recursion
locals[1] = Some(Number(10)) // param "n"

4. Frame pushed, execution continues from fn.ip = 0
```

### Closure Capture

At compile time, the compiler analyzes which variables from parent scopes
are used inside a function body:

```javascript
function outer(x) {
  function inner(y) {
    return x + y; // "x" is captured from outer
  }
  return inner(5);
}
```

Compiled:
```
outer:
captured_slots = [] // outer captures nothing
param_offset = 1 // slot 0 = self
slots: [0=self, 1=x]

inner:
captured_slots = [(1, 2)] // parent_slot=1 (x), new_slot=2
param_offset = 1
slots: [0=self, 1=y, 2=x(captured)]
```

At runtime, `MakeClosure` reads captured values from the current frame's locals
and stores them in the `BytecodeClosure.captured` vector.

---

## Value Types

```rust
enum Value {
  Number(f64),
  Bool(bool),
  Str(String),
  Array(Vec<Value>),
  Object(HashMap<String, Value>),
  Closure(Rc<Closure>), // Interpreter-era (not used by VM)
  Promise(Rc<RefCell<PromiseState>>),
  Resolver(Rc<RefCell<PromiseState>>),
  Rejector(Rc<RefCell<PromiseState>>),
  BytecodeClosure(BytecodeClosure), // VM closure
  Builtin(String), // "fetch", "console.log", etc.
  Expect { value: Box<Value>, negate: bool }, // Test framework
}
```

---

## Built-in Functions

### Console
- `console.log(...args)` — prints to stderr, returns `true`

### Fetch
- `fetch(url)` — HTTP GET, returns `Promise<Response>`
- `fetch(url, options)` — HTTP with options (method, headers, body)

### HTTP
- `http.get(url)` — HTTP GET
- `http.post(url, body)` — HTTP POST with JSON body
- `http.put(url, body)` — HTTP PUT
- `http.delete(url)` — HTTP DELETE
- `http.patch(url, body)` — HTTP PATCH
- `http.request(url, options)` — Universal HTTP
- `http.createServer(handler).listen(port)` — Start server

### Test Framework
- `test(name, fn)` — Run test, print ✓/✗
- `expect(value)` — Create expect object
- `describe(name, fn)` — Group tests

### Response Methods
- `res.json()` — Parse body as JSON
- `res.text()` — Get body as string

---

## CLI Architecture

### Commands

| Command | Implementation |
|---------|---------------|
| `djs run [file]` | `run_vm_file()` — compile + execute |
| `djs build [entry] [output]` | `build_project()` — bundle modules |
| `djs deploy [entry]` | `deploy_project()` — check + lint + build |
| `djs test [path]` | `run_test_files()` — execute test files |
| `djs check [path]` | `check_files()` — verify compilation |
| `djs lint [path]` | `lint_project()` — check purity |
| `djs repl` | `repl()` — interactive mode |

### Entry Point Auto-detection

```rust
fn find_entry() -> String {
  // 1. djs.json → "entry" field
  // 2. Common filenames: main.js, index.js, app.js, server.js
  // 3. src/ variants: src/main.js, src/index.js, src/app.js
}
```

### Deploy Pipeline

```
djs deploy
│
├─ Step 1: Check (djs run entry.js)
│ └─ Verify file compiles and executes
│
├─ Step 2: Lint (djs lint entry.js)
│ └─ Check for impure functions
│
└─ Step 3: Build (djs build entry.js)
  └─ Bundle all modules into dist/bundle.js
```

---

## Module System

### Resolution

Import paths are resolved relative to the importing file's directory:

```
project/
├── main.js import { add } from "./math"
├── math.js export function add(a, b) { ... }
└── utils/
  └── helpers.js import { add } from "../math"
```

- Relative paths (`./`, `../`) resolved from the importing file
- `.js` extension added automatically if missing
- Recursive dependency loading with cycle detection

### Compilation

The module system works by **source concatenation**:

1. Parse main file to find imports
2. Recursively load all imported modules
3. Strip `import` statements from all sources
4. Strip `export` prefixes from function declarations
5. Concatenate: `[imported sources] + [main source]`
6. Compile as a single program

---

## Test Suites

| Suite | Command | Tests | Description |
|-------|---------|-------|-------------|
| **Language** | `bash tests/language/test.sh` | 117 | VM features, operators, closures, modules |
| **CLI** | `bash tests/cli/test_check.sh` | 23 | run/build/check/lint/deploy/repl commands |
| **Bundler** | `bash tests/bundler/test_bundler.sh` | 9 | Module bundling and execution |
| **Linter** | `bash tests/checker/test_linter.sh` | 6 | Purity checks |
| **Test Framework** | `djs test tests/test_framework/` | 9 | expect/toEqual/not/comparison tests |
| **Total** | | **164** | |

---

## Key Design Decisions

### 1. Flat locals per frame (not linked environments)

Each `CallFrame` has a flat `Vec<Option<Value>>` for locals, not a chain of
parent environments. This makes variable lookup O(1) instead of O(depth).

**Tradeoff**: Compiler must compute exact slot layout at compile time, including
captured variables and destructured params.

### 2. Captured values by slot index, not by name

The compiler tracks `(parent_slot, new_slot)` pairs for each captured variable.
At runtime, `MakeClosure` copies values directly from the parent frame's locals.

**Tradeoff**: No name-based lookup at runtime, but compiler must correctly
analyze captures across nested function boundaries.

### 3. Promises executed synchronously

In this VM, `new Promise(executor)` calls the executor immediately and
`.then(callback)` invokes the callback synchronously if the promise is
already fulfilled. This is sufficient for the language's semantics
(no async/await, no event loop).

**Tradeoff**: No true asynchrony, but much simpler implementation.

### 4. Single value stack + call frames

The VM uses one shared `value_stack: Vec<Value>` for operand storage,
separate from the `frames: Vec<CallFrame>` call stack. This is the classic
stack-based VM design (like CPython, CRuby).

**Tradeoff**: Simple and efficient, but requires careful stack discipline.

### 5. No garbage collector (Rust ownership)

Values are `Clone` types (numbers, strings, HashMaps via `Rc` where needed).
When a frame is popped, its locals are dropped. No tracing GC needed.

**Tradeoff**: Cloning on some operations (string concat, array spread).
A real GC would reduce allocations for long-lived data.

### 6. Module system via source concatenation

Instead of complex bytecode linking, the module system concatenates
source files (with imports stripped) and compiles as one program.

**Tradeoff**: Simple and correct, but recompiles everything on every run.

---

## Running

```bash
# Install
cargo build --release
cp target/release/djs /usr/local/bin/djs

# Execute
djs file.js
djs run file.js

# Deploy
djs deploy

# Build
djs build src/main.js

# Test
djs test tests/

# Lint
djs lint src/

# REPL
djs repl

# Help
djs help
```

## Release Information

- **Version**: v0.1.0
- **Binary**: `target/release/djs`
- **Installation**: `install.sh`
- **Repository**: https://github.com/zyakiZyaki/djs
- **Author**: Max She

**Note**: This project was developed entirely by Max She. No external AI assistance was used in the development of this codebase.
