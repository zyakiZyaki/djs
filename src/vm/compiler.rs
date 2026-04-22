// Compiler: AST → Bytecode
// Supports nested functions via parent scope chain

use crate::vm::opcode::*;
use crate::parser::{Expr, Stmt, FuncDecl, Param, BinOp};
use crate::values::Value;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ============================================================================
// Scope with parent chain for nested functions
// ============================================================================

#[derive(Debug, Clone)]
pub struct Scope {
    parent: Option<Rc<Scope>>,
    vars: HashMap<String, usize>,
    next_slot: usize,
}

impl Scope {
    pub fn new() -> Self {
        Scope { parent: None, vars: HashMap::new(), next_slot: 0 }
    }

    pub fn with_parent(parent: Rc<Scope>) -> Self {
        Scope { parent: Some(parent), vars: HashMap::new(), next_slot: 0 }
    }

    /// Create a new scope with the same parent chain but WITHOUT specified function names
    pub fn without_funcs(current: Rc<Scope>, exclude: &HashSet<String>) -> Self {
        let mut vars = HashMap::new();
        let mut next_slot = 0;
        // Copy current scope's vars except excluded function names
        for (name, slot) in &current.vars {
            if !exclude.contains(name) {
                vars.insert(name.clone(), *slot);
                next_slot = next_slot.max(slot + 1);
            }
        }
        // Keep parent chain
        Scope { parent: current.parent.clone(), vars, next_slot }
    }

    fn collect_vars(scope: &Rc<Scope>, vars: &mut HashMap<String, usize>, next_slot: &mut usize, exclude: &HashSet<String>) {
        if let Some(p) = &scope.parent {
            Self::collect_vars(p, vars, next_slot, exclude);
        }
        for (name, slot) in &scope.vars {
            if !exclude.contains(name) {
                vars.insert(name.clone(), *slot);
                *next_slot = (*next_slot).max(slot + 1);
            }
        }
    }

    fn declare(&mut self, name: &str) -> usize {
        let slot = self.next_slot;
        self.vars.insert(name.to_string(), slot);
        self.next_slot += 1;
        slot
    }

    fn declare_at(&mut self, name: &str, slot: usize) {
        self.vars.insert(name.to_string(), slot);
        self.next_slot = self.next_slot.max(slot + 1);
    }

    fn resolve(&self, name: &str) -> Option<usize> {
        if let Some(&slot) = self.vars.get(name) {
            Some(slot)
        } else if let Some(parent) = &self.parent {
            parent.resolve(name)
        } else {
            None
        }
    }
}

// ============================================================================
// Compiler
// ============================================================================

pub struct Compiler {
    pub functions: Vec<CompiledFunction>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { functions: Vec::new() }
    }

    /// Compile the top-level program: multiple function decls + trailing expression.
    pub fn compile_program(&mut self, source: &str) -> Result<(Chunk, Vec<FuncDecl>), String> {
        self.compile_program_inner(source, false)
    }

    /// Compile with mode: as_locals=true stores functions as locals instead of globals
    pub fn compile_program_with_mode(&mut self, source: &str, as_locals: bool) -> Result<(Chunk, Vec<FuncDecl>), String> {
        self.compile_program_inner(source, as_locals)
    }

    fn compile_program_inner(&mut self, source: &str, as_locals: bool) -> Result<(Chunk, Vec<FuncDecl>), String> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();

        let mut chunk = Chunk::new();
        let mut scope = Scope::new();
        let mut all_decls = Vec::new();

        // Parse all function declarations
        let mut parser = crate::parser::Parser::new(tokens.clone());
        while matches!(parser.peek(), crate::lexer::Token::Function) {
            let saved_pos = parser.pos;
            parser.consume();
            if matches!(parser.peek(), crate::lexer::Token::Ident(_)) {
                parser.pos = saved_pos;
                let decl = parser.parse_func_decl()?;
                all_decls.push(decl.clone());

                let fn_idx = self.compile_function_with_scope(&decl, Rc::new(Scope::new()))?;
                
                if as_locals {
                    let local_slot = scope.declare(&decl.name);
                    chunk.emit(OpCode::MakeClosure(fn_idx as u32));
                    chunk.emit(OpCode::StoreLocal(local_slot as u32));
                } else {
                    let global_idx = chunk.add_global(&decl.name);
                    chunk.emit(OpCode::MakeClosure(fn_idx as u32));
                    chunk.emit(OpCode::StoreGlobal(global_idx));
                }
            } else {
                parser.pos = saved_pos;
                break;
            }
        }

        // Parse trailing expression (globals allowed here)
        if !matches!(parser.peek(), crate::lexer::Token::EOF) {
            let expr = parser.parse_expr()?;
            self.compile_expr(&mut chunk, &mut scope, &expr)?;
        }

        chunk.emit(OpCode::Return);
        Ok((chunk, all_decls))
    }

    pub fn compile_function_with_scope(&mut self, decl: &FuncDecl, parent_scope: Rc<Scope>) -> Result<usize, String> {
        // nesting_depth: 0 if no parent scope, 1+ otherwise — compute BEFORE any moves
        let nesting_depth = if parent_scope.parent.is_some() { 1 } else { 0 };

        let mut chunk = Chunk::new();

        let param_count = decl.params.len();
        let mut rest_slot: Option<usize> = None;

        // For named functions, slot 0 is reserved for the function itself (recursion)
        // Params start at slot 1
        let param_offset = if decl.name.is_empty() { 0 } else { 1 };

        // Create scope with params FIRST (before capture analysis)
        let mut scope_mut = Scope::with_parent(parent_scope.clone());
        let mut destructured_params: Vec<(usize, Vec<String>)> = Vec::new();

        for (i, param) in decl.params.iter().enumerate() {
            match param {
                Param::Name(n) => { scope_mut.declare_at(n, param_offset + i); }
                Param::Destructure(fields) => {
                    let obj_slot = param_offset + i;
                    for (j, f) in fields.iter().enumerate() {
                        scope_mut.declare_at(f, obj_slot + j);
                    }
                    destructured_params.push((i, fields.clone()));
                }
                Param::Rest(n) => {
                    scope_mut.declare_at(n, param_offset + i);
                    rest_slot = Some(i);
                }
            }
        }

        // Add self name at slot 0 for recursion (named functions only)
        let self_name = decl.name.clone();
        if param_offset > 0 {
            scope_mut.vars.insert(self_name.clone(), 0);
        }

        // Calculate total param slots (including destructured fields)
        let mut total_param_slots = 0;
        for (i, param) in decl.params.iter().enumerate() {
            match param {
                Param::Name(_) | Param::Rest(_) => { total_param_slots = i + 1; }
                Param::Destructure(fields) => { total_param_slots = i + fields.len() + 1; }
            }
        }

        // Collect sibling function names — these should NOT be captured
        let mut sibling_names: HashSet<String> = HashSet::new();
        for stmt in &decl.body {
            if let Stmt::FuncDecl(nested) = stmt {
                sibling_names.insert(nested.name.clone());
            }
        }

        // Collect captured variables from body statements
        // IMPORTANT: use scope_mut (with params) as parent, not parent_scope
        // BUT: when resolving, prefer scope_mut.vars over parent chain
        let mut captured: Vec<String> = Vec::new();
        fn collect_from_stmts(stmts: &[Stmt], parent: &Scope, captured: &mut Vec<String>, skip: &str, skip_siblings: &HashSet<String>) {
            for stmt in stmts {
                match stmt {
                    Stmt::FuncDecl(nested) => {
                        // Also collect captures from nested function bodies
                        for s in &nested.body {
                            match s {
                                Stmt::Return(e) => collect_captures_from_expr(e, parent, captured, skip, skip_siblings),
                                Stmt::Expr(e) => collect_captures_from_expr(e, parent, captured, skip, skip_siblings),
                                Stmt::FuncDecl(inner) => {
                                    // Recurse into deeper nested functions
                                    for s2 in &inner.body {
                                        match s2 {
                                            Stmt::Return(e) => collect_captures_from_expr(e, parent, captured, skip, skip_siblings),
                                            Stmt::Expr(e) => collect_captures_from_expr(e, parent, captured, skip, skip_siblings),
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Stmt::Return(e) => collect_captures_from_expr(e, parent, captured, skip, skip_siblings),
                    Stmt::Expr(e) => collect_captures_from_expr(e, parent, captured, skip, skip_siblings),
                    Stmt::Import(_) | Stmt::Export(_) => {}
                }
            }
        }
        fn collect_captures_from_expr(expr: &Expr, parent: &Scope, captured: &mut Vec<String>, skip: &str, skip_siblings: &HashSet<String>) {
            match expr {
                Expr::Ident(name) => {
                    if name != skip && !skip_siblings.contains(name) && parent.resolve(name).is_some() && !captured.contains(name) {
                        captured.push(name.clone());
                    }
                }
                Expr::Binary(l, _, r) => { collect_captures_from_expr(l, parent, captured, skip, skip_siblings); collect_captures_from_expr(r, parent, captured, skip, skip_siblings); }
                Expr::Conditional(c, t, e) => { collect_captures_from_expr(c, parent, captured, skip, skip_siblings); collect_captures_from_expr(t, parent, captured, skip, skip_siblings); collect_captures_from_expr(e, parent, captured, skip, skip_siblings); }
                Expr::Call(f, args) => { collect_captures_from_expr(f, parent, captured, skip, skip_siblings); for a in args { collect_captures_from_expr(a, parent, captured, skip, skip_siblings); } }
                Expr::ArrayLiteral(elems) => { for e in elems { collect_captures_from_expr(e, parent, captured, skip, skip_siblings); } }
                Expr::ObjectLiteral(entries) => { for (_, v) in entries { collect_captures_from_expr(v, parent, captured, skip, skip_siblings); } }
                Expr::Index(base, idx) => { collect_captures_from_expr(base, parent, captured, skip, skip_siblings); collect_captures_from_expr(idx, parent, captured, skip, skip_siblings); }
                Expr::PropAccess(obj, _) => { collect_captures_from_expr(obj, parent, captured, skip, skip_siblings); }
                Expr::MethodCall(obj, _, args) => { collect_captures_from_expr(obj, parent, captured, skip, skip_siblings); for a in args { collect_captures_from_expr(a, parent, captured, skip, skip_siblings); } }
                Expr::Function(_, body) => {
                    // For nested anonymous functions, don't skip the current function's name
                    // because it might be needed for recursion (e.g., step calling step)
                    for stmt in body {
                        if let crate::parser::Stmt::Return(e) = stmt {
                            collect_captures_from_expr(e, parent, captured, "", skip_siblings);
                        }
                        if let crate::parser::Stmt::Expr(e) = stmt {
                            collect_captures_from_expr(e, parent, captured, "", skip_siblings);
                        }
                    }
                }
                Expr::New(ctor, args) => { collect_captures_from_expr(ctor, parent, captured, skip, skip_siblings); for a in args { collect_captures_from_expr(a, parent, captured, skip, skip_siblings); } }
                Expr::Spread(inner) => { collect_captures_from_expr(inner, parent, captured, skip, skip_siblings); }
                Expr::Str(_) | Expr::Number(_) => {}
            }
        }
        collect_from_stmts(&decl.body, &scope_mut, &mut captured, &self_name, &sibling_names);

        // Set up captured vars at slots after params
        // Only add vars that aren't already in scope_mut (params or previously declared)
        let mut captured_slots = Vec::new();
        let mut captured_idx = 0;
        for cap in captured.iter() {
            // Check if var is already in scope_mut (either a param or already-declared var)
            if scope_mut.vars.get(cap).is_some() {
                // Variable is already accessible - skip adding to captured_slots
                // but we still need to track it for nested functions
                continue;
            }
            
            let parent_slot = scope_mut.resolve(cap).unwrap_or(0);
            let new_slot = param_offset + total_param_slots + captured_idx;
            captured_slots.push((parent_slot, new_slot));
            scope_mut.vars.insert(cap.clone(), new_slot);
            scope_mut.next_slot = scope_mut.next_slot.max(new_slot + 1);
            captured_idx += 1;
        }

        // Pre-declare nested function names before compilation
        // This allows nested functions to capture each other for recursion
        for stmt in &decl.body {
            if let Stmt::FuncDecl(nested) = stmt {
                scope_mut.declare(&nested.name);
            }
        }

        // Find the last Return statement index
        let last_return_idx = decl.body.iter().rposition(|s| matches!(s, Stmt::Return(_)));
        
        for (i, stmt) in decl.body.iter().enumerate() {
            match stmt {
                Stmt::FuncDecl(nested_decl) => {
                    // Create filtered scope WITHOUT OTHER siblings (but keep this function's name)
                    // so nested functions can capture it for recursion
                    let mut other_siblings = sibling_names.clone();
                    other_siblings.remove(&nested_decl.name);
                    let filtered_scope = Rc::new(Scope::without_funcs(Rc::new(scope_mut.clone()), &other_siblings));
                    let fn_idx = self.compile_function_with_scope(nested_decl, filtered_scope)?;
                    let slot = scope_mut.vars[&nested_decl.name];  // Already declared
                    chunk.emit(OpCode::MakeClosure(fn_idx as u32));
                    chunk.emit(OpCode::StoreLocal(slot as u32));
                }
                Stmt::Return(expr) => {
                    self.compile_expr(&mut chunk, &mut scope_mut, expr)?;
                    // Only emit Return for the last return; others just drop the value
                    if last_return_idx.map(|idx| idx == i).unwrap_or(false) {
                        chunk.emit(OpCode::Return);
                    } else {
                        chunk.emit(OpCode::Pop);
                    }
                }
                Stmt::Expr(expr) => {
                    // Expression statement: evaluate but don't return
                    self.compile_expr(&mut chunk, &mut scope_mut, expr)?;
                    chunk.emit(OpCode::Pop);
                }
                Stmt::Import(_) | Stmt::Export(_) => {}
            }
        }

        // If no return was found, emit one anyway
        if last_return_idx.is_none() {
            // Push undefined
            let idx = chunk.add_constant(Value::Bool(true));
            chunk.emit(OpCode::PushConst(idx));
            chunk.emit(OpCode::Return);
        }

        let fn_idx = self.functions.len();
        self.functions.push(CompiledFunction {
            chunk,
            param_count,
            name: decl.name.clone(),
            captured_slots,
            rest_slot,
            destructured_params,
            param_offset,
            nesting_depth,
        });
        Ok(fn_idx)
    }

    fn compile_stmt(&mut self, chunk: &mut Chunk, scope: &mut Scope, stmt: &Stmt, sibling_names: &HashSet<String>) -> Result<(), String> {
        match stmt {
            Stmt::FuncDecl(decl) => {
                // Collect sibling function names only (not all vars)
                let filtered_scope = Rc::new(Scope::without_funcs(Rc::new(scope.clone()), sibling_names));
                let fn_idx = self.compile_function_with_scope(decl, filtered_scope)?;
                let slot = scope.declare(&decl.name);
                chunk.emit(OpCode::MakeClosure(fn_idx as u32));
                chunk.emit(OpCode::StoreLocal(slot as u32));
            }
            Stmt::Return(expr) => {
                self.compile_expr(chunk, scope, expr)?;
                chunk.emit(OpCode::Return);
            }
            Stmt::Expr(expr) => {
                // Expression statement: evaluate but don't return
                self.compile_expr(chunk, scope, expr)?;
                chunk.emit(OpCode::Pop);
            }
            Stmt::Import(_) | Stmt::Export(_) => {
                // Imports are handled at the module level, exports are just named functions
                // No bytecode needed — these are processed before compilation
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, chunk: &mut Chunk, scope: &mut Scope, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Number(n) => {
                let idx = chunk.add_constant(Value::Number(*n));
                chunk.emit(OpCode::PushConst(idx));
            }
            Expr::Str(s) => {
                let idx = chunk.add_constant(Value::Str(s.clone()));
                chunk.emit(OpCode::PushConst(idx));
            }
            Expr::Ident(name) => {
                if let Some(slot) = scope.resolve(name) {
                    chunk.emit(OpCode::LoadLocal(slot as u32));
                } else {
                    let idx = chunk.add_global(name);
                    chunk.emit(OpCode::LoadGlobal(idx));
                }
            }
            Expr::Binary(left, op, right) => {
                match op {
                    BinOp::And => {
                        self.compile_expr(chunk, scope, left)?;
                        chunk.emit(OpCode::Dup);
                        let jump_idx = chunk.code.len();
                        chunk.emit(OpCode::JumpIfFalsePop(0));
                        chunk.emit(OpCode::Pop);
                        self.compile_expr(chunk, scope, right)?;
                        let target = chunk.code.len();
                        chunk.patch_jump(jump_idx, target);
                        chunk.emit(OpCode::ToBool);
                    }
                    BinOp::Or => {
                        self.compile_expr(chunk, scope, left)?;
                        chunk.emit(OpCode::Dup);
                        let jump_idx = chunk.code.len();
                        chunk.emit(OpCode::JumpIfTruePop(0));
                        chunk.emit(OpCode::Pop);
                        self.compile_expr(chunk, scope, right)?;
                        let target = chunk.code.len();
                        chunk.patch_jump(jump_idx, target);
                        chunk.emit(OpCode::ToBool);
                    }
                    _ => {
                        self.compile_expr(chunk, scope, left)?;
                        self.compile_expr(chunk, scope, right)?;
                        match op {
                            BinOp::Add => chunk.emit(OpCode::Add),
                            BinOp::Sub => chunk.emit(OpCode::Sub),
                            BinOp::Mul => chunk.emit(OpCode::Mul),
                            BinOp::Div => chunk.emit(OpCode::Div),
                            BinOp::Mod => chunk.emit(OpCode::Mod),
                            BinOp::Eq => chunk.emit(OpCode::Eq),
                            BinOp::Ne => chunk.emit(OpCode::Ne),
                            BinOp::Lt => chunk.emit(OpCode::Lt),
                            BinOp::Gt => chunk.emit(OpCode::Gt),
                            BinOp::Le => chunk.emit(OpCode::Le),
                            BinOp::Ge => chunk.emit(OpCode::Ge),
                            _ => {}
                        }
                    }
                }
            }
            Expr::Conditional(cond, then_expr, else_expr) => {
                self.compile_expr(chunk, scope, cond)?;
                let jump_idx = chunk.code.len();
                chunk.emit(OpCode::JumpIfFalsePop(0));
                self.compile_expr(chunk, scope, then_expr)?;
                let end_jump = chunk.code.len();
                chunk.emit(OpCode::Jump(0));
                let else_target = chunk.code.len();
                chunk.patch_jump(jump_idx, else_target);
                self.compile_expr(chunk, scope, else_expr)?;
                let end_target = chunk.code.len();
                chunk.patch_jump(end_jump, end_target);
            }
            Expr::Call(func, args) => {
                self.compile_expr(chunk, scope, func)?;
                // Count actual runtime arguments (spread expands to N args)
                let mut arg_count = 0u16;
                for arg in args {
                    if let Expr::Spread(inner) = arg {
                        self.compile_expr(chunk, scope, inner)?;
                        chunk.emit(OpCode::Spread);
                        // We can't know array size at compile time, so we'll use a special opcode
                        // For now, increment by 1 and VM will handle it
                        arg_count += 1;
                    } else {
                        self.compile_expr(chunk, scope, arg)?;
                        arg_count += 1;
                    }
                }
                // For spread calls, we need special handling
                // Use negative count to signal spread expansion
                let has_spread = args.iter().any(|a| matches!(a, Expr::Spread(_)));
                if has_spread {
                    // Count non-spread args + marker for spread
                    let non_spread = args.iter().filter(|a| !matches!(a, Expr::Spread(_))).count() as u16;
                    chunk.emit(OpCode::CallSpread(non_spread));
                } else {
                    chunk.emit(OpCode::Call(arg_count));
                }
            }
            Expr::Function(params, body) => {
                // Compile anonymous function: params get slots 0..N-1
                // Captured variables get slots N..N+M-1 (after params)
                let param_count = params.len();

                // Collect captured variable names
                let mut captured: Vec<String> = Vec::new();
                fn collect_captures(expr: &Expr, parent: &Scope, captured: &mut Vec<String>) {
                    match expr {
                        Expr::Ident(name) => {
                            if parent.resolve(name).is_some() && !captured.contains(name) {
                                captured.push(name.clone());
                            }
                        }
                        Expr::Binary(l, _, r) => { collect_captures(l, parent, captured); collect_captures(r, parent, captured); }
                        Expr::Conditional(c, t, e) => { collect_captures(c, parent, captured); collect_captures(t, parent, captured); collect_captures(e, parent, captured); }
                        Expr::Call(f, args) => { collect_captures(f, parent, captured); for a in args { collect_captures(a, parent, captured); } }
                        Expr::ArrayLiteral(elems) => { for e in elems { collect_captures(e, parent, captured); } }
                        Expr::ObjectLiteral(entries) => { for (_, v) in entries { collect_captures(v, parent, captured); } }
                        Expr::Index(base, idx) => { collect_captures(base, parent, captured); collect_captures(idx, parent, captured); }
                        Expr::PropAccess(obj, _) => { collect_captures(obj, parent, captured); }
                        Expr::MethodCall(obj, _, args) => { collect_captures(obj, parent, captured); for a in args { collect_captures(a, parent, captured); } }
                        Expr::Function(_, body) => {
                            // Recurse into nested function body to collect captures
                            for stmt in body {
                                match stmt {
                                    crate::parser::Stmt::Return(e) => {
                                        collect_captures(e, parent, captured);
                                    }
                                    crate::parser::Stmt::Expr(e) => {
                                        collect_captures(e, parent, captured);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Expr::New(ctor, args) => { collect_captures(ctor, parent, captured); for a in args { collect_captures(a, parent, captured); } }
                        Expr::Spread(inner) => { collect_captures(inner, parent, captured); }
                        _ => {}
                    }
                }
                for stmt in body {
                    match stmt {
                        crate::parser::Stmt::Return(e) => {
                            collect_captures(e, scope, &mut captured);
                        }
                        crate::parser::Stmt::Expr(e) => {
                            collect_captures(e, scope, &mut captured);
                        }
                        crate::parser::Stmt::FuncDecl(decl) => {
                            // Also collect captures from nested function bodies
                            for s in &decl.body {
                                match s {
                                    crate::parser::Stmt::Return(e) => {
                                        collect_captures(e, scope, &mut captured);
                                    }
                                    crate::parser::Stmt::Expr(e) => {
                                        collect_captures(e, scope, &mut captured);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Also collect captures from EXPR functions inside Return statements
                fn collect_from_return_expr(expr: &Expr, scope: &Scope, captured: &mut Vec<String>) {
                    if let Expr::Function(params, body) = expr {
                        // Collect from this function's body using the OUTER scope
                        for stmt in body {
                            if let crate::parser::Stmt::Return(e) = stmt {
                                collect_captures(e, scope, captured);
                                // Recurse into nested Expr::Function
                                collect_from_return_expr(e, scope, captured);
                            }
                        }
                    }
                }
                for stmt in body {
                    if let crate::parser::Stmt::Return(e) = stmt {
                        collect_from_return_expr(e, scope, &mut captured);
                    }
                }

                let parent_scope = Rc::new(scope.clone());
                let mut fn_chunk = Chunk::new();
                let mut fn_scope = Scope::with_parent(parent_scope);

                // Detect rest param and destructured params
                let mut anonymous_rest_slot: Option<usize> = None;
                let mut anonymous_destructured: Vec<(usize, Vec<String>)> = Vec::new();

                // Step 1: params at 0..N-1
                for (i, param) in params.iter().enumerate() {
                    match param {
                        Param::Name(n) => { fn_scope.declare(n); }
                        Param::Destructure(fields) => {
                            let slot = fn_scope.declare(&fields[0]);
                            for f in fields.iter().skip(1) {
                                fn_scope.declare(f);
                            }
                            anonymous_destructured.push((i, fields.clone()));
                        }
                        Param::Rest(n) => {
                            let slot = fn_scope.declare(n);
                            anonymous_rest_slot = Some(i);  // Store param index, not slot
                        }
                    }
                }

                // Step 2: captured vars after all param slots
                // Calculate total param slots (including destructured fields)
                let mut anon_total_param_slots = 0;
                for (i, param) in params.iter().enumerate() {
                    match param {
                        Param::Name(_) | Param::Rest(_) => { anon_total_param_slots = i + 1; }
                        Param::Destructure(fields) => { anon_total_param_slots = i + fields.len() + 1; }
                    }
                }
                
                let mut captured_slots = Vec::new();
                for (i, cap) in captured.iter().enumerate() {
                    let parent_slot = scope.resolve(cap).unwrap_or(0);
                    let new_slot = anon_total_param_slots + i;
                    captured_slots.push((parent_slot, new_slot));
                    fn_scope.vars.insert(cap.clone(), new_slot);
                    fn_scope.next_slot = fn_scope.next_slot.max(new_slot + 1);
                }

                for stmt in body {
                    self.compile_stmt(&mut fn_chunk, &mut fn_scope, stmt, &HashSet::new())?;
                }
                if fn_chunk.code.is_empty() || !matches!(fn_chunk.code.last(), Some(OpCode::Return)) {
                    fn_chunk.emit(OpCode::Return);
                }

                let fn_idx = self.functions.len();
                self.functions.push(CompiledFunction {
                    chunk: fn_chunk,
                    param_count,
                    name: "<anonymous>".to_string(),
                    captured_slots,
                    rest_slot: anonymous_rest_slot,
                    destructured_params: anonymous_destructured,
                    param_offset: 0,
                    nesting_depth: 1, // anonymous functions are always nested
                });

                chunk.emit(OpCode::MakeClosure(fn_idx as u32));
            }
            Expr::ArrayLiteral(elems) => {
                chunk.emit(OpCode::PushDepthMarker);
                for elem in elems {
                    if let Expr::Spread(inner) = elem {
                        self.compile_expr(chunk, scope, inner)?;
                        chunk.emit(OpCode::Spread);
                    } else {
                        self.compile_expr(chunk, scope, elem)?;
                    }
                }
                chunk.emit(OpCode::MakeArray);
            }
            Expr::ObjectLiteral(entries) => {
                for (key, val) in entries {
                    let key_idx = chunk.add_constant(Value::Str(key.clone()));
                    chunk.emit(OpCode::PushConst(key_idx));
                    self.compile_expr(chunk, scope, val)?;
                }
                chunk.emit(OpCode::MakeObject(entries.len() as u16));
            }
            Expr::Index(base, idx) => {
                self.compile_expr(chunk, scope, base)?;
                self.compile_expr(chunk, scope, idx)?;
                chunk.emit(OpCode::Index);
            }
            Expr::PropAccess(obj, prop) => {
                self.compile_expr(chunk, scope, obj)?;
                let prop_idx = chunk.add_property(prop);
                chunk.emit(OpCode::GetProperty(prop_idx));
            }
            Expr::MethodCall(obj, method, args) => {
                self.compile_expr(chunk, scope, obj)?;
                let method_idx = chunk.add_constant(Value::Str(method.clone()));
                chunk.emit(OpCode::PushConst(method_idx));
                for arg in args {
                    self.compile_expr(chunk, scope, arg)?;
                }
                chunk.emit(OpCode::MethodCall(args.len() as u16));
            }
            Expr::Spread(_) => {}
            Expr::New(ctor, args) => {
                if let Expr::Ident(name) = ctor.as_ref() {
                    if name == "Promise" && !args.is_empty() {
                        // new Promise(executor) — compile executor call with 2 implicit args
                        self.compile_expr(chunk, scope, &args[0])?;
                        // We'll use a special opcode to create promise and call executor
                        chunk.emit(OpCode::MakePromise);
                        return Ok(());
                    }
                }
                self.compile_expr(chunk, scope, ctor)?;
                for arg in args {
                    self.compile_expr(chunk, scope, arg)?;
                }
                chunk.emit(OpCode::Call(args.len() as u16));
            }
        }
        Ok(())
    }
}
