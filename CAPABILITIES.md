# DJS — Declarative JavaScript

> A strict, declarative subset of JavaScript with built-in test framework, bundler, and linter.

## Overview

DJS (Declarative JavaScript) is a **pure-functional, immutable, declarative subset of JavaScript**. Every valid DJS program is also valid JavaScript and runs in any JS engine (V8, Bun, Node.js).

### Core Principles

**Pure Immutability** — DJS has no mutable state and no OOP:

- ❌ No `let`, `var`, `const` — all bindings are function parameters
- ❌ No assignment operators — no reassignment ever
- ❌ No `if/else`, `for`, `while` — use ternary and recursion
- ❌ No `class`, `this`, `new` (except Promise) — objects are pure data
- ✅ Pure functions only — same input → same output, no side effects
- ✅ Immutable data — never mutated, only transformed into new values

### Key Features

- **JavaScript-compatible syntax** — familiar syntax for JS developers
- **Declarative style** — describe what, not how
- **Pure functions** — no side effects, predictable behavior
- **Immutable data** — data is never mutated, only transformed
- **First-class functions** — closures, currying, higher-order functions
- **Built-in test framework** — `test()`, `expect()`, `describe()` (no imports)
- **Module system** — `import`/`export` for code organization
- **HTTP API** — fetch and http server for network requests
- **CLI tools** — `djs run`, `djs build`, `djs deploy`, `djs test`, `djs lint`

---

## Quick Start

### Install

```bash
# Build from source
cargo build --release
cp target/release/djs /usr/local/bin/djs

# Verify
djs version
# DJS v0.1.0 (Declarative JavaScript VM)
```

### Create a Project

```
my-project/
├── djs.json              # project config (optional)
├── src/
│   └── main.js           # entry point
├── modules/
│   └── math.js           # local modules
└── tests/
    └── math_test.js      # test files
```

```json
// djs.json
{
  "name": "my-project",
  "version": "0.1.0",
  "entry": "src/main.js",
  "modules": "./modules"
}
```

### Run

```bash
djs run src/main.js       # run a file
djs run                   # auto-detects entry from djs.json or common names
```

### Deploy (check + lint + build)

```bash
djs deploy                # one command to production
# 🚀 DJS Deploy
#   Entry: src/main.js
# Step 1/3: Checking files... ✓
# Step 2/3: Checking purity... ✓
# Step 3/3: Building bundle... ✓
# ✅ Deployed successfully in 0.007s
#   Bundle: dist/bundle.js
#   Size: 1234 bytes
```

### Test

```bash
djs test tests/           # run all tests
djs test tests/math.js    # run single test
```

---

## Syntax

### Functions

```javascript
// Named function
function add(a, b) {
    return a + b;
}

// Nested functions
function outer(x) {
    function inner(y) {
        return x + y;
    }
    return inner(10);
}

// Anonymous function (returned or passed)
function makeMultiplier(factor) {
    return function(x) {
        return x * factor;
    };
}
```

### Closures

Functions capture variables from outer scopes:

```javascript
function counter(eff) {
    function step(count) {
        return {
            inc: function () {
                eff(count + 1);
                return step(count + 1);
            },
            dec: function () {
                eff(count - 1);
                return step(count - 1);
            }
        };
    }
    return step;
}
```

### Destructuring

```javascript
// Object destructuring in parameters
function getName({name}) {
    return name;
}

function getBoth({name, age}) {
    return {name: name, age: age};
}
```

### Rest Params & Spread

```javascript
// Rest params
function sum(...args) {
    return args.reduce(function(acc, x) { return acc + x; }, 0);
}

// Spread
function combine(a, b) {
    return [...a, ...b];
}
```

### Arrays & Objects

```javascript
// Arrays
function process(arr) {
    return arr
        .map(function(x) { return x * 2; })
        .filter(function(x) { return x > 5; })
        .reduce(function(acc, x) { return acc + x; }, 0);
}

// Objects
function createPerson(name, age) {
    return {name: name, age: age};
}

// Object methods
function getKeys(obj) {
    return obj.keys();
}
```

### Ternary (no if/else)

```javascript
function abs(x) {
    return x < 0 ? -x : x;
}

function classify(n) {
    return n < 0 ? "negative"
        : n == 0 ? "zero"
        : "positive";
}
```

### Promises

```javascript
new Promise(function(resolve, reject) {
    resolve(42);
}).then(function(x) {
    return x * 2;
});
// → 84
```

### Modules

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

**Important:** In DJS, imports must be passed as parameters to functions. Functions cannot capture global imports directly — this enforces purity.

```javascript
// ❌ IMPURE: function captures global import
import { add } from "./math";
function test() { return add(1, 2); }

// ✅ PURE: dependency passed as parameter
import { add } from "./math";
function test(fn) { return fn(1, 2); }
test(add)
```

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `djs [file.js]` | Run a DJS file |
| `djs run [file.js]` | Run (auto-detects entry from djs.json) |
| `djs build [entry.js] [output.js]` | Bundle project → single file |
| `djs deploy [entry.js]` | Check + lint + build in one command |
| `djs test [path]` | Run test files with built-in test framework |
| `djs check [path]` | Verify files compile without running |
| `djs lint [path]` | Check code purity (detect impure functions) |
| `djs repl` | Start interactive REPL |
| `djs version` | Show version |
| `djs help` | Show help |

### `djs deploy` — One Command to Production

Runs three steps in sequence:
1. **Check** — verifies entry file compiles and runs
2. **Lint** — checks for impure functions
3. **Build** — bundles all modules into `dist/bundle.js`

```bash
djs deploy              # auto-detects entry (djs.json or main.js/index.js)
djs deploy src/main.js  # explicit entry
```

### `djs build` — Bundler

Recursively resolves all imports and bundles into a single file:

```bash
djs build src/main.js           # → dist/bundle.js
djs build src/main.js out.js    # → out.js
```

### `djs lint` — Purity Checker

Detects functions that capture imports without taking them as parameters:

```bash
djs lint src/
# ✓ All clear! 5 files checked

djs lint dirty.js
# ✗ 1 error(s):
#   ✗ dirty.js: function 'test' captures global import 'add' (impure function)
```

---

## Built-in Test Framework

No imports needed — `test`, `expect`, and `describe` are globals.

```javascript
describe("math", function() {
    test("addition", function() {
        expect(1 + 2).toEqual(3);
    });
    test("not equal", function() {
        expect(1).not.toEqual(2);
    });
    test("greater than", function() {
        expect(5).toBeGreaterThan(3);
    });
    test("truthy", function() {
        expect(1).toBeTruthy();
    });
});
```

### Matchers

| Matcher | Description |
|---------|-------------|
| `expect(a).toEqual(b)` | Deep equality (numbers, strings, arrays, objects) |
| `expect(a).toBe(b)` | Alias for toEqual |
| `expect(a).not.toEqual(b)` | Negation |
| `expect(a).toBeTruthy()` | Checks value is truthy |
| `expect(a).toBeFalsy()` | Checks value is falsy |
| `expect(a).toBeGreaterThan(b)` | Number comparison |
| `expect(a).toBeLessThan(b)` | Number comparison |

### Organization

| Function | Description |
|----------|-------------|
| `test(name, fn)` | Run a test, print ✓ or ✗ |
| `describe(name, fn)` | Group tests with a header |

### Running Tests

```bash
djs test tests/             # run all .js files in directory
djs test tests/math.js      # run single test file
```

Output:
```
── math ──
  ✓ addition
  ✓ not equal
  ✓ greater than
  ✓ truthy
```

---

## HTTP API

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
    return res.status;  // → 201
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
// Server running on http://127.0.0.1:8888
```

The handler **must** be a closure defined inside another function (nesting_depth > 0).

### HTTP Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `fetch(url)` | Promise<Response> | HTTP GET |
| `fetch(url, opts)` | Promise<Response> | HTTP with options |
| `http.get(url)` | Promise<Response> | HTTP GET |
| `http.post(url, body)` | Promise<Response> | HTTP POST |
| `http.put(url, body)` | Promise<Response> | HTTP PUT |
| `http.delete(url)` | Promise<Response> | HTTP DELETE |
| `http.patch(url, body)` | Promise<Response> | HTTP PATCH |
| `http.request(url, opts)` | Promise<Response> | Universal HTTP |
| `http.createServer(fn).listen(port)` | — | Start HTTP server |

### Response Object

| Property/Method | Description |
|-----------------|-------------|
| `res.status` | HTTP status code (200, 201, 404, etc.) |
| `res.ok` | Boolean: 200-299 |
| `res.headers` | Response headers object |
| `res.json()` | Parse body as JSON |
| `res.text()` | Get body as string |

---

## Built-in Methods

### Array

| Method | Description | Example |
|--------|-------------|---------|
| `arr.map(fn)` | Transform each element | `[1,2].map(fn)` → `[2,4]` |
| `arr.filter(fn)` | Filter elements | `[1,2,3].filter(fn)` → `[2,3]` |
| `arr.reduce(fn, init)` | Reduce to single value | `[1,2,3].reduce(fn, 0)` → `6` |
| `arr.push(x)` | Add element | `[1].push(2)` → `[1,2]` |
| `arr.concat(arr)` | Concatenate arrays | `[1].concat([2])` → `[1,2]` |
| `arr.join(sep)` | Join to string | `[1,2].join("-")` → `"1-2"` |
| `arr.length` | Get length | `[1,2].length` → `2` |

### Object

| Method | Description | Example |
|--------|-------------|---------|
| `obj.keys()` | Get all keys | `{a:1}.keys()` → `["a"]` |
| `obj.values()` | Get all values | `{a:1}.values()` → `[1]` |

### String

| Method | Description | Example |
|--------|-------------|---------|
| `str.length` | Get length | `"hello".length` → `5` |
| `str.split(sep)` | Split into array | `"a,b".split(",")` → `["a","b"]` |

### JSON

| Method | Description |
|--------|-------------|
| `JSON.parse(str)` | Parse JSON string → value |
| `JSON.stringify(val)` | Serialize value → JSON string |

### Console

| Method | Description |
|--------|-------------|
| `console.log(...args)` | Print to stderr |

---

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `2 + 3` → `5` |
| `-` | Subtraction | `5 - 2` → `3` |
| `*` | Multiplication | `3 * 4` → `12` |
| `/` | Division | `10 / 2` → `5` |
| `%` | Modulo | `7 % 3` → `1` |

### Comparison

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `5 == 5` → `true` |
| `!=` | Not equal | `5 != 3` → `true` |
| `<` | Less than | `3 < 5` → `true` |
| `>` | Greater than | `5 > 3` → `true` |
| `<=` | Less or equal | `5 <= 5` → `true` |
| `>=` | Greater or equal | `5 >= 3` → `true` |

### Logical

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `true && true` → `true` |
| `\|\|` | Logical OR | `true \|\| false` → `true` |
| `? :` | Ternary | `true ? 1 : 0` → `1` |

---

## Project Structure

```
my-project/
├── djs.json              # Project config (optional)
│   {
│     "name": "my-project",
│     "entry": "src/main.js",
│     "modules": "./modules"
│   }
├── src/
│   └── main.js           # Entry point
├── modules/              # Local modules
│   ├── math.js
│   └── utils.js
├── tests/
│   ├── unit/             # Unit tests
│   ├── integration/      # Integration tests
│   └── test_framework/   # Test framework tests
└── dist/
    └── bundle.js         # Generated by djs build/deploy
```

### Entry Point Auto-detection

When no file is specified, DJS looks for:
1. `djs.json` → `"entry"` field
2. `main.js`, `index.js`, `app.js`, `server.js`
3. `src/main.js`, `src/index.js`, `src/app.js`

---

## Test Suites

DJS has 5 test suites (164 total tests):

| Suite | Command | Tests | Description |
|-------|---------|-------|-------------|
| **Language** | `bash tests/language/test.sh` | 117 | VM features, operators, closures, modules |
| **CLI** | `bash tests/cli/test_check.sh` | 23 | run/build/check/lint/deploy/repl commands |
| **Bundler** | `bash tests/bundler/test_bundler.sh` | 9 | Module bundling and execution |
| **Linter** | `bash tests/checker/test_linter.sh` | 6 | Purity checks |
| **Test Framework** | `djs test tests/test_framework/` | 9 | expect/toEqual/not/comparison tests |

Run all:
```bash
bash tests/language/test.sh
bash tests/cli/test_check.sh
bash tests/bundler/test_bundler.sh
bash tests/checker/test_linter.sh
djs test tests/test_framework/
```

---

## Running

```bash
# Install
cargo build --release
cp target/release/djs /usr/local/bin/djs

# Development
djs run                    # auto-detect entry
djs test tests/            # run tests
djs lint src/              # check purity

# Production
djs deploy                 # check + lint + build
```
